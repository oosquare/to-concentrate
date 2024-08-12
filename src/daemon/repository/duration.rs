use std::sync::Arc;

use crate::daemon::config::{Configuration, ConfigurationContent};
use crate::domain::entity::StageDuration;
use crate::domain::repository::{duration::GetDurationError, DurationRepository};

/// A [`DurationRepository`] implementation which reads configuration files.
pub struct DurationConfiguration {
    config: Arc<Configuration>,
}

impl DurationConfiguration {
    /// Creates a new [`DurationConfiguration`].
    pub fn new(config: Arc<Configuration>) -> Self {
        Self { config }
    }

    /// Return the configurations or wrap the error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the internal `Result` is `Err`.
    fn get(&self) -> Result<&ConfigurationContent, GetDurationError> {
        self.config.get().map_err(|err| GetDurationError::Unknown {
            message: "Could not load configurations".to_owned(),
            source: Some(err.into()),
        })
    }
}

#[async_trait::async_trait]
impl DurationRepository for DurationConfiguration {
    async fn preparation_duration(&self) -> Result<StageDuration, GetDurationError> {
        let content = self.get()?;
        let raw = content.duration.preparation;
        let value = raw
            .try_into()
            .map_err(|err| GetDurationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn concentration_duration(&self) -> Result<StageDuration, GetDurationError> {
        let content = self.get()?;
        let raw = content.duration.concentration;
        let value = raw
            .try_into()
            .map_err(|err| GetDurationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn relaxation_duration(&self) -> Result<StageDuration, GetDurationError> {
        let content = self.get()?;
        let raw = content.duration.relaxation;
        let value = raw
            .try_into()
            .map_err(|err| GetDurationError::Invalid { source: err })?;
        Ok(value)
    }
}
