use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Arguments {
    /// Path to a custom configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    /// Whether to daemonize the process
    #[arg(short, long)]
    pub daemonize: bool,
}
