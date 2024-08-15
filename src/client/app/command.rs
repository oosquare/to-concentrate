#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Launch and initialize a daemon process
    Init,
    /// Pause the timer
    Pause,
    /// Resume the timer
    Resume,
    /// Query the timer's status. Show all information if no flag is specified.
    Query(QueryArguments),
    /// Skip the current stage
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryArguments {
    /// Show the timer's current status
    pub current: bool,
    /// Show the current stage's name
    pub stage: bool,
    /// Show the total duration in the current stage
    pub total: bool,
    /// Show the remaining duration in the current stage
    pub remaining: bool,
    /// Show the past duration in the current stage
    pub past: bool,
}
