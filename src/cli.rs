use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Divera Status Tracker", long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// Config path
    #[arg(global = true, short, long)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands of the application
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the config
    Init(Init),
}

#[derive(Debug, Args)]
pub struct Init {
    /// Accesskey for divera247
    #[arg(long)]
    pub access_key: String,
}
