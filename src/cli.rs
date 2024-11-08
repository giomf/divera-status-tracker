use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Divera Status Tracker", long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// Config path
    #[arg(global = true, long)]
    pub config: Option<String>,

    /// Data path
    #[arg(global = true, long)]
    pub data_dir: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands of the application
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the config
    Init(Init),

    /// Fetch current status
    Update,

    /// Prints the data
    Print(Print),
}

#[derive(Debug, Args)]
pub struct Init {
    /// Accesskey for divera247
    #[arg(long)]
    pub access_key: String,
}

#[derive(Debug, Args)]
pub struct Print {
    /// The year to print
    #[arg(long)]
    pub year: Option<i32>,
}
