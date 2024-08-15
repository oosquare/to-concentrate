use std::path::PathBuf;

use clap::{Parser, Subcommand};
use to_concentrate::client::app::{Command as ClientCommand, QueryArguments};
use tracing::Level;

#[derive(Debug, Parser)]
pub struct Arguments {
    /// Path to a custom configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch and initialize a daemon process
    Init {
        /// Maximum logging level the subscriber should use
        #[arg(short, long, default_value_t = Level::INFO)]
        verbosity: Level,
    },
    /// Pause the timer
    Pause,
    /// Resume the timer
    Resume,
    /// Query the timer's status. Show all information if no flag is specified.
    Query {
        /// Show the current stage's name
        #[arg(short, long)]
        stage: bool,
        /// Show the total duration in the current stage
        #[arg(short, long)]
        total: bool,
        /// Show the remaining duration in the current stage
        #[arg(short, long)]
        remaining: bool,
        /// Show the past duration in the current stage
        #[arg(short, long)]
        past: bool,
    },
    /// Skip the current stage
    Skip,
}

impl From<Command> for ClientCommand {
    fn from(value: Command) -> Self {
        match value {
            Command::Init { .. } => Self::Init,
            Command::Pause => Self::Pause,
            Command::Resume => Self::Resume,
            Command::Query {
                stage,
                total,
                remaining,
                past,
            } => Self::Query(QueryArguments {
                stage,
                total,
                remaining,
                past,
            }),
            Command::Skip => Self::Skip,
        }
    }
}
