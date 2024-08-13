use std::sync::Arc;

use crate::daemon::config::Configuration;
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
}

#[async_trait::async_trait]
impl NotificationRepository for NotificationConfiguration {
    async fn preparation_notification(&self) -> Result<NotificationMessage, GetNotificationError> {
        let section = self.config.notification.preparation.clone();
        let value = NotificationMessage::try_new(section.summary, section.body)
            .map_err(|err| GetNotificationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn concentration_notification(
        &self,
    ) -> Result<NotificationMessage, GetNotificationError> {
        let section = self.config.notification.concentration.clone();
        let value = NotificationMessage::try_new(section.summary, section.body)
            .map_err(|err| GetNotificationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn relaxation_notification(&self) -> Result<NotificationMessage, GetNotificationError> {
        let section = self.config.notification.relaxation.clone();
        let value = NotificationMessage::try_new(section.summary, section.body)
            .map_err(|err| GetNotificationError::Invalid { source: err })?;
        Ok(value)
    }
}
