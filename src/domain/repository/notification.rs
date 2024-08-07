use std::error::Error as StdError;

use snafu::prelude::*;

use crate::domain::entity::notification::{NotificationMessage, TryNewNotificationMessageError};

/// An abstract interface for accessing an notification's information.
#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait NotificationRepository: Send + Sync + 'static {
    /// Get the message for post-[`Preparation`] notification.
    ///
    /// [`Preparation`]: crate::domain::entity::state::StageState::Preparation
    ///
    /// # Errors
    ///
    /// This function will return an error if failed to get the message.
    async fn preparation_notification(&self) -> Result<NotificationMessage, GetNotificationError>;

    /// Get the message for post-[`Concentration`] notification.
    ///
    /// [`Concentration`]: crate::domain::entity::state::StageState::Concentration
    ///
    /// # Errors
    ///
    /// This function will return an error if failed to get the message.
    async fn concentration_notification(&self)
        -> Result<NotificationMessage, GetNotificationError>;

    /// Get the message for post-[`Relaxation`] notification.
    ///
    /// [`Relaxation`]: crate::domain::entity::state::StageState::Relaxation
    ///
    /// # Errors
    ///
    /// This function will return an error if failed to get the message.
    async fn relaxation_notification(&self) -> Result<NotificationMessage, GetNotificationError>;
}

/// An error type of accessing the repository of [`NotificationMessage`]s.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum GetNotificationError {
    #[snafu(display("Could not create an invalid notification message"))]
    #[non_exhaustive]
    Invalid {
        source: TryNewNotificationMessageError,
    },
    #[snafu(whatever, display("Could not get message of notification: {message}"))]
    #[non_exhaustive]
    Unknown {
        message: String,
        #[snafu(source(from(Box<dyn StdError>, Some)))]
        source: Option<Box<dyn StdError>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn notification_repository_get() {
        let mock = init_mock();
        assert_eq!(
            mock.preparation_notification().await.unwrap(),
            NotificationMessage::try_new("summary".into(), Some("body".into())).unwrap()
        );
    }

    fn init_mock() -> MockNotificationRepository {
        let mut mock = MockNotificationRepository::new();
        mock.expect_preparation_notification().return_once(|| {
            Ok(NotificationMessage::try_new("summary".into(), Some("body".into())).unwrap())
        });
        mock
    }
}
