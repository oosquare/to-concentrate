use std::error::Error as StdError;

use snafu::prelude::*;

use crate::domain::entity::notification::NotificationMessage;

/// A public port for emitting a notification.
pub trait NotifyPort: Send + Sync + 'static {
    /// Do the notification operation.
    async fn notify(&self, request: NotifyRequest) -> Result<(), NotifyError>;
}

/// A structure that stores required data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyRequest {
    pub summary: String,
    pub body: Option<String>,
}

impl From<NotificationMessage> for NotifyRequest {
    fn from(value: NotificationMessage) -> Self {
        let (summary, body) = value.into();
        Self { summary, body }
    }
}

/// An error type of the notification operation.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum NotifyError {
    #[snafu(
        whatever,
        display("An unknown error occurred while making a notification.")
    )]
    Unknown {
        message: String,
        #[snafu(source(from(Box<dyn StdError>, Some)))]
        source: Option<Box<dyn StdError>>,
    },
}
