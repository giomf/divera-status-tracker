use std::{fmt::Display, fs, path::Path, sync::LazyLock};

use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use either::Either;
use polars::prelude::*;

use crate::UserStatus;

const NAME_COLUMN_NAME: &str = "Name";
const DATE_COLUMN_NAME: &str = "Date";
const TOTAL_ON_DUTY_COLUMN_NAME: &str = "Total On-Duty";
const WEEKEND_ON_DUTY_COLUMN_NAME: &str = "Weekend On-Duty";
const WEEKEND_BEGIN_HOUR: i8 = 18;
const WEEKEND_BEGIN_DAY: i8 = 5;
const DATE_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

static STATE: LazyLock<Expr> =
    LazyLock::new(|| col("*").exclude([NAME_COLUMN_NAME, DATE_COLUMN_NAME]));
static DATE: LazyLock<Expr> = LazyLock::new(|| {
    col(DATE_COLUMN_NAME)
        .str()
        .to_datetime(None, None, StrptimeOptions::default(), lit("raise"))
});

#[derive(Debug, Default, Clone)]
pub struct Data {
    data: DataFrame,
}

impl Data {
    pub fn from_parquet(path: &Path) -> Result<Self> {
        let mut file = std::fs::File::open(path).context("Failed to open file")?;
        let data = ParquetReader::new(&mut file)
            .finish()
            .context("Failed to read parquet file")?;

        Ok(Self { data })
    }

    pub fn write_parquet(&mut self, file: &Path) -> Result<()> {
        let mut file = fs::File::create(file).context("Failed to create file")?;

        ParquetWriter::new(&mut file)
            .finish(&mut self.data)
            .context("Failed to write to aggragation file")?;
        Ok(())
    }

    pub fn append(&mut self, datetime: NaiveDateTime, users: &Vec<UserStatus>) -> Result<()> {
        let names: Vec<String> = users
            .iter()
            .map(|user| format!("{} {}", user.firstname, user.lastname).to_string())
            .collect();
        let status: Vec<String> = users.iter().map(|user| user.status.to_string()).collect();
        let names = Column::new(NAME_COLUMN_NAME.into(), names);
        let status = Column::new(datetime.format(DATE_TIME_FORMAT).to_string().into(), status);

        let new_data = DataFrame::new(vec![names, status])?;
        log::debug!("New data to append {}", new_data);

        if self.data.is_empty() {
            self.data = new_data;
            return Ok(());
        }

        self.data = self.data.join(
            &new_data,
            [NAME_COLUMN_NAME],
            [NAME_COLUMN_NAME],
            JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
        )?;

        Ok(())
    }

    pub fn calculate(&self) -> Result<DataFrame> {
        let total_on_duty = self
            .calculate_total_on_duty()
            .context("Failed to calculate total on duty")?;
        let weekend_on_duty = self
            .calculate_weekend_on_duty()
            .context("Failed to calculate weekend on duty")?;

        log::debug!("{TOTAL_ON_DUTY_COLUMN_NAME}: {}", total_on_duty);
        log::debug!("{WEEKEND_ON_DUTY_COLUMN_NAME}: {}", weekend_on_duty);

        let sort_options = SortMultipleOptions::new()
            .with_multithreaded(true)
            .with_order_descending(true);

        let result = total_on_duty
            .lazy()
            .join(
                weekend_on_duty.lazy(),
                [col(NAME_COLUMN_NAME)],
                [col(NAME_COLUMN_NAME)],
                JoinArgs::new(JoinType::Left),
            )
            .collect()?
            .sort([TOTAL_ON_DUTY_COLUMN_NAME], sort_options)?;

        log::debug!("Result: {}", result);
        Ok(result)
    }

    fn calculate_total_on_duty(&self) -> Result<DataFrame> {
        let names = self.get_names().context("Failed to get names")?;
        let total_on_duty = self
            .data
            .clone()
            .lazy()
            .select([STATE.clone()])
            .collect()?
            .transpose(Some(DATE_COLUMN_NAME), Some(Either::Right(names)))?
            .lazy()
            .select([STATE
                .clone()
                .eq(lit("On Duty"))
                .sum()
                .cast(DataType::Float32)
                / STATE.clone().count()
                * lit(100)])
            .collect()?
            .transpose(Some(NAME_COLUMN_NAME), None)?
            .rename("column_0", TOTAL_ON_DUTY_COLUMN_NAME.into())?
            .clone();

        Ok(total_on_duty)
    }

    fn calculate_weekend_on_duty(&self) -> Result<DataFrame> {
        let names = self.get_names().context("Failed to get names")?;

        let weekend_expression = DATE
            .clone()
            .dt()
            .weekday()
            .gt(lit(WEEKEND_BEGIN_DAY))
            .or(DATE
                .clone()
                .dt()
                .weekday()
                .eq(lit(WEEKEND_BEGIN_DAY))
                .and(DATE.clone().dt().hour().gt_eq(lit(WEEKEND_BEGIN_HOUR))));

        let weekend_on_duty_expression = STATE
            .clone()
            .eq(lit("On Duty"))
            .and(weekend_expression.clone())
            .sum();

        let weekend_on_duty = self
            .data
            .clone()
            .lazy()
            .select([STATE.clone()])
            .collect()?
            .transpose(Some(DATE_COLUMN_NAME), Some(Either::Right(names)))?
            .lazy()
            .select([(weekend_on_duty_expression.cast(DataType::Float32)
                / weekend_expression.sum()
                * lit(100))])
            .collect()?
            .transpose(Some(NAME_COLUMN_NAME), None)?
            .rename("column_0", WEEKEND_ON_DUTY_COLUMN_NAME.into())?
            .clone();

        Ok(weekend_on_duty)
    }

    fn get_names(&self) -> Result<Vec<String>> {
        let names: Vec<String> = self
            .data
            .column("Name")?
            .as_materialized_series()
            .iter()
            .map(|name| name.str_value().to_string())
            .collect();
        Ok(names)
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.data.to_string())
    }
}
