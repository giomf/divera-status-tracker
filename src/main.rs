mod cli;
mod config;
mod data;

use anyhow::{bail, Context, Result};
use chrono::offset::Utc;
use clap::Parser;
use config::Config;
use data::Data;
use divera;
use env_logger;
use std::{collections::HashMap, fmt::Display, path::Path};

use cli::{Cli, Commands};

const CONFIG_PATH: &str = "./config.toml";
const DATA_PATH: &str = "./data.parquet";

#[derive(Debug, Clone)]
struct UserStatus {
    pub firstname: String,
    pub lastname: String,
    pub status: Status,
}

#[derive(Debug, Clone)]
enum Status {
    OnDuty,
    OffDuty,
}
impl UserStatus {
    pub fn new_user_status(
        users: &Vec<divera::v1::schema::response::User>,
        status_descriptions: &HashMap<i64, divera::v2::schema::response::Status>,
    ) -> Result<Vec<UserStatus>> {
        let users = users
            .iter()
            .map(|user| {
                let status_description = status_descriptions
                    .get(&user.status_id)
                    .context(format!("Status id {} not found", user.status_id))
                    .unwrap();

                let status = Status::new(status_description, "Außer Dienst");

                UserStatus {
                    firstname: user.firstname.clone(),
                    lastname: user.lastname.clone(),
                    status,
                }
            })
            .collect();

        Ok(users)
    }
}
impl Status {
    pub fn new(status: &divera::v2::schema::response::Status, off_duty_keyword: &str) -> Self {
        if status.name == off_duty_keyword {
            return Status::OffDuty;
        }
        return Status::OnDuty;
    }
}
impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::OnDuty => f.write_str("On Duty"),
            Status::OffDuty => f.write_str("Off Duty"),
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or(CONFIG_PATH.to_string());
    let config_path = Path::new(&config_path);
    let data_path = cli.data.unwrap_or(DATA_PATH.to_string());
    let data_path = Path::new(&data_path);

    match cli.command {
        Commands::Init(arguments) => {
            if config_path.exists() {
                bail!("Config already exists. Aborting");
            }
            let config = Config::new(&arguments.access_key);
            config
                .write(config_path)
                .context("Failed to write config")?;
        }
        Commands::Update => {
            let config = Config::read(config_path).context("Failed to read config")?;

            let users =
                divera::v1::users(&config.divera.access_key).context("Failed to fetch users")?;
            let all =
                divera::v2::pull_all(&config.divera.access_key).context("Failed to fetch all")?;
            let status = all.cluster.status;
            let users = UserStatus::new_user_status(&users, &status)?;
            let date = Utc::now().naive_utc();

            let mut data = if data_path.exists() {
                Data::from_parquet(data_path).context("Failed to read data from parquet")?
            } else {
                Data::default()
            };

            data.append(date, &users).context("Failed to append data")?;
            data.write_parquet(data_path)
                .context("Failed to write data")?;
        }
        Commands::Print => {
            if !data_path.exists() {
                bail!(
                    "Path {} does not exists",
                    data_path.to_string_lossy().to_string()
                );
            }
            let data = Data::from_parquet(data_path).context("Failed to read data from parquet")?;
            let data = data.calculate().context("Failed to calculate data")?;
            println!("{data}");
        }
    };
    Ok(())
}
