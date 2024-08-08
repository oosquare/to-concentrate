use std::fmt::{Display, Formatter, Result as FmtResult};

/// The state of the working procedure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageState {
    Preparation,
    Concentration,
    Relaxation,
}

impl StageState {
    /// Get an initialized [`StageState`].
    pub fn initial() -> Self {
        Self::Preparation
    }

    /// Get the next [`StageState`] based on the current one.
    pub fn next(self) -> Self {
        match self {
            Self::Preparation => Self::Concentration,
            Self::Concentration => Self::Relaxation,
            Self::Relaxation => Self::Concentration,
        }
    }
}

impl Display for StageState {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Preparation => f.write_str("Preparation"),
            Self::Concentration => f.write_str("Concentration"),
            Self::Relaxation => f.write_str("Relaxation"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_next() {
        let state = StageState::initial();
        assert_eq!(state, StageState::Preparation);
        let state = state.next();
        assert_eq!(state, StageState::Concentration);
        let state = state.next();
        assert_eq!(state, StageState::Relaxation);
        let state = state.next();
        assert_eq!(state, StageState::Concentration);
        let state = state.next();
        assert_eq!(state, StageState::Relaxation);
    }
}
