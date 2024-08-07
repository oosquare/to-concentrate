use std::error::Error as StdError;

use snafu::prelude::*;

use crate::domain::entity::notification::{NotificationMessage, TryNewNotificationMessageError};

/// An abstract interface for accessing an notification's information.
#[cfg_attr(test, mockall::automock)]
pub trait NotificationRepository: Send + Sync + 'static {
    async fn notification_message(&self) -> Result<NotificationMessage, GetNotificationError>;
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
            mock.notification_message().await.unwrap(),
            NotificationMessage::try_new("summary".into(), Some("body".into())).unwrap()
        );
    }

    fn init_mock() -> MockNotificationRepository {
        let mut mock = MockNotificationRepository::new();
        mock.expect_notification_message().return_once(|| {
            Ok(NotificationMessage::try_new("summary".into(), Some("body".into())).unwrap())
        });
        mock
    }
}
