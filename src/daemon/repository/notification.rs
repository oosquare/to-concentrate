use std::sync::Arc;

use crate::daemon::config::{Configuration, ConfigurationContent};
use crate::domain::entity::NotificationMessage;
use crate::domain::repository::{notification::GetNotificationError, NotificationRepository};

/// A [`NotificationRepository`] implementation which reads configuration files.
pub struct NotificationConfiguration {
    config: Arc<Configuration>,
}

impl NotificationConfiguration {
    /// Creates a new [`NotificationConfiguration`].
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    /// Return the configurations or wrap the error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the internal `Result` is `Err`.
    fn get(&self) -> Result<&ConfigurationContent, GetNotificationError> {
        self.config
            .get()
            .map_err(|err| GetNotificationError::Unknown {
                message: "Could not load configurations".to_owned(),
                source: Some(err.into()),
            })
    }
}

#[async_trait::async_trait]
impl NotificationRepository for NotificationConfiguration {
    async fn preparation_notification(&self) -> Result<NotificationMessage, GetNotificationError> {
        let content = self.get()?;
        let summary = content.notification.preparation.summary.clone();
        let body = content.notification.preparation.body.clone();
        let value = NotificationMessage::try_new(summary, body)
            .map_err(|err| GetNotificationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn concentration_notification(
        &self,
    ) -> Result<NotificationMessage, GetNotificationError> {
        let content = self.get()?;
        let summary = content.notification.concentration.summary.clone();
        let body = content.notification.concentration.body.clone();
        let value = NotificationMessage::try_new(summary, body)
            .map_err(|err| GetNotificationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn relaxation_notification(&self) -> Result<NotificationMessage, GetNotificationError> {
        let content = self.get()?;
        let summary = content.notification.relaxation.summary.clone();
        let body = content.notification.relaxation.body.clone();
        let value = NotificationMessage::try_new(summary, body)
            .map_err(|err| GetNotificationError::Invalid { source: err })?;
        Ok(value)
    }
}
