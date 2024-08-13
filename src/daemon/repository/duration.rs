use std::sync::Arc;

use crate::daemon::config::Configuration;
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
}

#[async_trait::async_trait]
impl DurationRepository for DurationConfiguration {
    async fn preparation_duration(&self) -> Result<StageDuration, GetDurationError> {
        let raw = self.config.duration.preparation;
        let value = raw
            .try_into()
            .map_err(|err| GetDurationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn concentration_duration(&self) -> Result<StageDuration, GetDurationError> {
        let raw = self.config.duration.concentration;
        let value = raw
            .try_into()
            .map_err(|err| GetDurationError::Invalid { source: err })?;
        Ok(value)
    }

    async fn relaxation_duration(&self) -> Result<StageDuration, GetDurationError> {
        let raw = self.config.duration.relaxation;
        let value = raw
            .try_into()
            .map_err(|err| GetDurationError::Invalid { source: err })?;
        Ok(value)
    }
}
