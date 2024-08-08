use std::sync::Arc;

use snafu::prelude::*;

use crate::domain::daemon::app::service::{PauseService, QueryService, ResumeService, SkipService};
use crate::domain::daemon::inbound::{PausePort, QueryPort, ResumePort, SkipPort};
use crate::domain::daemon::outbound::NotifyPort;
use crate::domain::daemon::worker::{self, SpawnWorkerError};
use crate::domain::repository::{DurationRepository, NotificationRepository};

/// Entrance to the domain logic, providing ports for external adapters.
pub struct Application {
    pub pause: Arc<dyn PausePort>,
    pub resume: Arc<dyn ResumePort>,
    pub query: Arc<dyn QueryPort>,
    pub skip: Arc<dyn SkipPort>,
}

impl Application {
    /// Initialize the application by injecting external repositories and
    /// adapters.
    ///
    /// # Errors
    ///
    /// This function will return an error if initialization failed.
    pub async fn init(
        notify_port: Arc<dyn NotifyPort>,
        duration_repository: Arc<dyn DurationRepository>,
        notification_repository: Arc<dyn NotificationRepository>,
    ) -> Result<Application, InitApplicationError> {
        let worker = worker::spawn(duration_repository, notification_repository, notify_port)
            .await
            .context(WorkerSnafu)?;
        let worker = Arc::new(worker);

        let pause_port = Arc::new(PauseService::new(Arc::clone(&worker)));
        let resume_port = Arc::new(ResumeService::new(Arc::clone(&worker)));
        let query_port = Arc::new(QueryService::new(Arc::clone(&worker)));
        let skip_port = Arc::new(SkipService::new(Arc::clone(&worker)));

        let app = Application {
            pause: pause_port,
            resume: resume_port,
            query: query_port,
            skip: skip_port,
        };

        Ok(app)
    }
}

#[derive(Debug, Snafu)]
pub enum InitApplicationError {
    #[snafu(display("Could not spawn a background worker"))]
    Worker { source: SpawnWorkerError },
}
