use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toml;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub divera: Divera,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Divera {
    pub access_key: String,
}

impl Config {
    pub fn new(access_key: &str) -> Self {
        Config {
            divera: Divera {
                access_key: access_key.to_string(),
            },
        }
    }

    pub fn read(path: &Path) -> Result<Self> {
        let config = fs::read_to_string(path).context("Failed to read config")?;
        let config = toml::from_str(&config).context("Failed to parse config")?;
        log::debug!("Read config: {:#?}", config);
        Ok(config)
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let config = toml::to_string(&self).context("Failed to render config")?;
        fs::write(path, config).context("Failed to write config")?;
        Ok(())
    }
}
