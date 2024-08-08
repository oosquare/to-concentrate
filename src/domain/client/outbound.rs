use std::error::Error as StdError;

use snafu::prelude::*;
use tokio::time::Duration;

use crate::domain::daemon::inbound::QueryResponse;

/// A public port for launching and initializing a daemon.
pub trait InitPort {
    /// Do the initialization operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if daemon is already running or the
    /// initialization failed.
    async fn init(&self) -> Result<(), InitDaemonError>;
}

/// An error type of initializing a daemon.
#[derive(Debug, Snafu)]
pub enum InitDaemonError {
    #[snafu(display("Daemon is already running"))]
    AlreadyRunning,
    #[snafu(whatever, display("Initialization failed: {message}"))]
    Unknown {
        message: String,
        #[snafu(source(from(Box<dyn StdError>, Some)))]
        source: Option<Box<dyn StdError>>,
    },
}

/// A public port for requesting the daemon to suspend the tomato timer.
#[async_trait::async_trait]
pub trait PausePort {
    /// Do the pause operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the operation failed.
    async fn pause(&self) -> Result<(), RequestDaemonError>;
}

/// A public port for requesting the daemon to resume the tomato timer.
#[async_trait::async_trait]
pub trait ResumePort {
    /// Do the resume operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the operation failed.
    async fn resume(&self) -> Result<(), RequestDaemonError>;
}

/// A public port for requesting the daemon to query the current state.
#[async_trait::async_trait]
pub trait QueryPort: Send + Sync + 'static {
    /// Do the query operation.
    async fn query(&self) -> Result<QueryResponse, RequestDaemonError>;
}

/// A public port for requesting the daemon to skip the current stage.
#[async_trait::async_trait]
pub trait SkipPort {
    /// Do the skip operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the operation failed.
    async fn skip(&self) -> Result<(), RequestDaemonError>;
}

/// An error type of sending requests to daemon.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum RequestDaemonError {
    #[snafu(display("Daemon is unavailable on interface {interface}"))]
    Unavailable { interface: String },
    #[snafu(display("Could not receive a response in time"))]
    Timeout,
    #[snafu(whatever, display("Request failed: {message}"))]
    Unknown {
        message: String,
        #[snafu(source(from(Box<dyn StdError>, Some)))]
        source: Option<Box<dyn StdError>>,
    },
}
