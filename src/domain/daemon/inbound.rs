use tokio::time::Duration;

/// A public port for suspending the tomato timer.
pub trait PausePort: Send + Sync + 'static {
    /// Do the pause operation.
    async fn pause(&self);
}

/// A public port for resuming the tomato timer.
pub trait ResumePort: Send + Sync + 'static {
    // Do the resume operation.
    async fn resume(&self);
}

/// A public port for querying the current state.
pub trait QueryPort: Send + Sync + 'static {
    /// Do the query operation.
    async fn query(&self) -> QueryResponse;
}

/// The state of this daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResponse {
    stage: String,
    total: Duration,
    remaining: Duration,
    past: Duration,
}

/// A public port for skip the current stage.
pub trait SkipPort: Send + Sync + 'static {
    /// Do the skipping operation.
    async fn skip(&self);
}
