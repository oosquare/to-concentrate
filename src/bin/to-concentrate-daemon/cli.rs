use std::path::PathBuf;

use clap::Parser;
use tracing::Level;

#[derive(Debug, Parser)]
pub struct Arguments {
    /// Path to a custom configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    /// Maximum logging level the subscriber should use
    #[arg(short, long, default_value_t = Level::INFO)]
    pub verbosity: Level,
    /// Whether to daemonize the process
    #[arg(short, long)]
    pub daemonize: bool,
}
