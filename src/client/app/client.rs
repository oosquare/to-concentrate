use std::sync::Arc;

use snafu::prelude::*;

use crate::client::app::command::{Command, QueryArguments};
use crate::domain::client::outbound::{InitDaemonError, RequestDaemonError};
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

    /// Run specific function according to `command`.
    ///
    /// # Errors
    ///
    /// This function will return an error if any error occurs.
    pub async fn run(&self, command: Command) -> Result<(), ClientError> {
        match command {
            Command::Init => self.init().await,
            Command::Pause => self.pause().await,
            Command::Resume => self.resume().await,
            Command::Query(args) => self.query(args).await,
            Command::Skip => self.skip().await,
        }
    }

    /// Send `init` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the daemon fails to be launched.
    async fn init(&self) -> Result<(), ClientError> {
        self.core.init.init().await.context(InitDaemonSnafu)
    }

    /// Send `pause` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    async fn pause(&self) -> Result<(), ClientError> {
        self.core.pause.pause().await.context(RequestSnafu)
    }

    /// Send `resume` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    async fn resume(&self) -> Result<(), ClientError> {
        self.core.resume.resume().await.context(RequestSnafu)
    }

    /// Send `query` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    async fn query(&self, args: QueryArguments) -> Result<(), ClientError> {
        let response = self.core.query.query().await.context(RequestSnafu)?;
        let enable_all =
            !args.current && !args.stage && !args.total && !args.remaining && !args.past;
        let mut outputs = Vec::new();

        if enable_all || args.current {
            outputs.push(("Current".to_owned(), response.current));
        }

        if enable_all || args.stage {
            outputs.push(("Stage".to_owned(), response.stage));
        }

        if enable_all || args.total {
            let value = format!("{}s", response.total.as_secs().to_string());
            outputs.push(("Total".to_owned(), value));
        }

        if enable_all || args.remaining {
            let value = format!("{}s", response.remaining.as_secs().to_string());
            outputs.push(("Remaining".to_owned(), value));
        }

        if enable_all || args.past {
            let value = format!("{}s", response.past.as_secs().to_string());
            outputs.push(("Past".to_owned(), value));
        }

        let key_align = outputs
            .iter()
            .map(|(key, _)| key.len())
            .max()
            .unwrap_or_default();

        for (mut key, value) in outputs {
            let padding = " ".to_owned().repeat(key_align - key.len());
            key.push_str(&padding);
            println!("{key} = {value}");
        }

        Ok(())
    }

    /// Send `skip` request to daemon.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client fails to receive a
    /// valid response.
    async fn skip(&self) -> Result<(), ClientError> {
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
