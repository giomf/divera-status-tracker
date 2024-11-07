mod cli;
mod config;

use anyhow::{Context, Result};
use clap::Parser;
use config::Config;
use env_logger;
use std::path::Path;

use cli::{Cli, Commands};

pub const CONFIG_PATH: &str = "./config.toml";

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or(CONFIG_PATH.to_string());
    let config_path = Path::new(&config_path);

    match cli.command {
        Commands::Init(arguments) => {
            let config = Config::new(&arguments.access_key);
            config
                .write(config_path)
                .context("Failed to write config")?;
        }
    };
    Ok(())
}
