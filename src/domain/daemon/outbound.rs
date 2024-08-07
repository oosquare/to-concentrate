use std::error::Error as StdError;

use snafu::prelude::*;

use crate::domain::entity::notification::NotificationMessage;

/// A public port for emitting a notification.
#[async_trait::async_trait]
pub trait NotifyPort: Send + Sync + 'static {
    /// Do the notification operation. This method is not intended to be
    /// implemented by adapters directly.
    ///
    /// # Errors
    ///
    /// This function will return an error if failed to make a notification.
    async fn notify(&self, request: &NotificationMessage) -> Result<(), NotifyError> {
        let request = NotifyRequest {
            summary: request.summary().to_owned(),
            body: request.body().map(|body| body.to_owned()),
        };
        self.notify_impl(request).await
    }

    /// Actual implementation of notification operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if failed to make a notification.
    async fn notify_impl(&self, request: NotifyRequest) -> Result<(), NotifyError>;
}

/// A structure that stores required data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyRequest {
    pub summary: String,
    pub body: Option<String>,
}

/// An error type of the notification operation.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum NotifyError {
    #[snafu(whatever, display("Could not emit a notification: {message}"))]
    Unknown {
        message: String,
        #[snafu(source(from(Box<dyn StdError>, Some)))]
        source: Option<Box<dyn StdError>>,
    },
}
