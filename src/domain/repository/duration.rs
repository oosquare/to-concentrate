use std::error::Error as StdError;

use snafu::prelude::*;

use crate::domain::entity::duration::StageDuration;

/// An abstract interface for accessing duration data.
#[cfg_attr(test, mockall::automock)]
pub trait DurationRepository: Send + Sync + 'static {
    /// Get duration of the [`Preparation`] stage.
    ///
    /// [`Preparation`]: crate::domain::entity::state::StageState::Preparation
    async fn preparation_duration(&self) -> Result<StageDuration, GetDurationError>;

    /// Get duration of the [`Concentration`] stage.
    ///
    /// [`Concentration`]: crate::domain::entity::state::StageState::Concentration
    async fn concentration_duration(&self) -> Result<StageDuration, GetDurationError>;

    /// Get duration of the [`Relaxation`] stage.
    ///
    /// [`Relaxation`]: crate::domain::entity::state::StageState::Relaxation
    async fn relaxation_duration(&self) -> Result<StageDuration, GetDurationError>;
}

/// An error type of accessing the repository of [`StageDuration`]s.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum GetDurationError {
    #[snafu(whatever, display("an internal error occurred."))]
    #[non_exhaustive]
    Internal {
        message: String,
        #[snafu(source(from(Box<dyn StdError>, Some)))]
        source: Option<Box<dyn StdError>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn duration_repository_get() {
        let mock = init_mock();

        assert_eq!(
            mock.preparation_duration().await.unwrap(),
            StageDuration::try_new(10).unwrap()
        );

        assert!(mock.concentration_duration().await.is_err());
    }

    fn init_mock() -> MockDurationRepository {
        let mut mock = MockDurationRepository::new();
        mock.expect_preparation_duration()
            .returning(|| Ok(StageDuration::try_new(10).unwrap()));
        mock.expect_concentration_duration()
            .returning(|| whatever!("error"));
        mock
    }
}