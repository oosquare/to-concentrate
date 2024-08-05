use std::time::Duration;

use snafu::prelude::*;

/// The duration of each stage represented in seconds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StageDuration(Duration);

impl StageDuration {
    /// Try to create a [`StageDuration`] from a u64 integer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the integer is zero.
    pub fn try_new(seconds: u64) -> Result<Self, TryNewStageDurationError> {
        ensure!(seconds > 0, ZeroSnafu);
        Ok(Self(Duration::from_secs(seconds)))
    }

    /// Returns a reference to the inner of this [`StageDuration`].
    pub fn inner(&mut self) -> &Duration {
        &self.0
    }
}

impl TryFrom<u64> for StageDuration {
    type Error = TryNewStageDurationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// An error type of creating a [`StageDuration`].
#[derive(Debug, Clone, Snafu, PartialEq, Eq)]
#[non_exhaustive]
pub enum TryNewStageDurationError {
    #[snafu(display("Duration must be greater than zero"))]
    #[non_exhaustive]
    Zero,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_duration_try_new() {
        assert_eq!(
            StageDuration::try_new(10),
            Ok(StageDuration(Duration::from_secs(10))),
        );
        assert_eq!(
            StageDuration::try_new(0),
            Err(TryNewStageDurationError::Zero),
        );
    }

    #[test]
    fn stage_duration_try_from() {
        assert_eq!(10.try_into(), Ok(StageDuration(Duration::from_secs(10))),);
        assert_eq!(
            0.try_into(),
            Err::<StageDuration, TryNewStageDurationError>(TryNewStageDurationError::Zero)
        );
    }
}
