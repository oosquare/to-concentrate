use std::sync::Arc;

use snafu::prelude::*;

use crate::domain::client::outbound::{InitDaemonError, QueryResponse, RequestDaemonError};
use crate::domain::client::ApplicationCore;

/// Main business logic implementation in client side.
pub struct Client {
    core: Arc<ApplicationCore>,
}

impl Client {
    /// Creates a new [`Client`].
    pub fn new(core: Arc<ApplicationCore>) -> Self {
        Self { core }
    }

    /// Send `init` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the daemon fails to be launched.
    pub async fn init(&self) -> Result<(), ClientError> {
        self.core.init.init().await.context(InitDaemonSnafu)
    }

    /// Send `pause` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    pub async fn pause(&self) -> Result<(), ClientError> {
        self.core.pause.pause().await.context(RequestSnafu)
    }

    /// Send `resume` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    pub async fn resume(&self) -> Result<(), ClientError> {
        self.core.resume.resume().await.context(RequestSnafu)
    }

    /// Send `query` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    pub async fn query(&self) -> Result<QueryResponse, ClientError> {
        self.core.query.query().await.context(RequestSnafu)
    }

    /// Send `skip` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    pub async fn skip(&self) -> Result<(), ClientError> {
        self.core.skip.skip().await.context(RequestSnafu)
    }
}

/// An error for client's operations.
#[derive(Debug, Snafu)]
pub enum ClientError {
    #[snafu(display("Could not initialize daemon"))]
    InitDaemon { source: InitDaemonError },
    #[snafu(display("Could request daemon"))]
    Request { source: RequestDaemonError },
}
