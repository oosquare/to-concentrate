use std::sync::Arc;

use crate::domain::client::outbound::{InitPort, PausePort, QueryPort, ResumePort, SkipPort};

/// Entrance to the domain logic, providing ports for external adapters.
pub struct ApplicationCore {
    pub init: Arc<dyn InitPort>,
    pub pause: Arc<dyn PausePort>,
    pub resume: Arc<dyn ResumePort>,
    pub query: Arc<dyn QueryPort>,
    pub skip: Arc<dyn SkipPort>,
}

impl ApplicationCore {
    /// Create and initialize a new [`ApplicationCore`] by injecting external
    /// repositories and adapters.
    pub fn setup(
        init: Arc<dyn InitPort>,
        pause: Arc<dyn PausePort>,
        resume: Arc<dyn ResumePort>,
        query: Arc<dyn QueryPort>,
        skip: Arc<dyn SkipPort>,
    ) -> ApplicationCore {
        Self {
            init,
            pause,
            resume,
            query,
            skip,
        }
    }
}
