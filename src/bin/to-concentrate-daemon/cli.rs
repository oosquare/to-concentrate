use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Arguments {
    /// Path to a custom configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    /// Path where the daemon creates the UNIX socket
    #[arg(short, long)]
    pub socket: Option<PathBuf>,
    /// Whether to daemonize the process
    #[arg(short, long)]
    pub daemonize: bool,
}
