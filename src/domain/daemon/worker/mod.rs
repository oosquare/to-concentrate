mod handle;
mod routine;
mod state;

pub use handle::{QueryResponse, WorkerHandle};

use std::sync::Arc;

use snafu::prelude::*;

use crate::domain::daemon::outbound::NotifyPort;
use crate::domain::entity::StageState;
use crate::domain::repository::duration::{DurationRepository, GetDurationError};
use crate::domain::repository::notification::{GetNotificationError, NotificationRepository};

use routine::{WorkerConfig, WorkerRoutine};

pub async fn spawn(
    duration_repository: Arc<dyn DurationRepository>,
    notification_repository: Arc<dyn NotificationRepository>,
    notifier: Arc<dyn NotifyPort>,
) -> Result<WorkerHandle, SpawnWorkerError> {
    let (requester, commands) = tokio::sync::mpsc::channel(1);
    let config = load_config(duration_repository, notification_repository).await?;
    let handle = WorkerRoutine::spawn(config, commands, notifier);
    Ok(WorkerHandle::new(requester, handle))
}

async fn load_config(
    duration_repository: Arc<dyn DurationRepository>,
    notification_repository: Arc<dyn NotificationRepository>,
) -> Result<WorkerConfig, SpawnWorkerError> {
    let preparation_duration =
        duration_repository
            .preparation_duration()
            .await
            .context(DurationConfigSnafu {
                key: StageState::Preparation,
            })?;
    let concentration_duration =
        duration_repository
            .concentration_duration()
            .await
            .context(DurationConfigSnafu {
                key: StageState::Concentration,
            })?;
    let relaxation_duration =
        duration_repository
            .relaxation_duration()
            .await
            .context(DurationConfigSnafu {
                key: StageState::Relaxation,
            })?;
    let preparation_notification = notification_repository
        .preparation_notification()
        .await
        .context(NotificationConfigSnafu {
            key: StageState::Preparation,
        })?;
    let concentration_notification = notification_repository
        .concentration_notification()
        .await
        .context(NotificationConfigSnafu {
            key: StageState::Concentration,
        })?;
    let relaxation_notification = notification_repository
        .relaxation_notification()
        .await
        .context(NotificationConfigSnafu {
            key: StageState::Relaxation,
        })?;

    Ok(WorkerConfig {
        preparation_duration,
        concentration_duration,
        relaxation_duration,
        preparation_notification,
        concentration_notification,
        relaxation_notification,
    })
}

/// An error for spawning the background worker.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum SpawnWorkerError {
    #[snafu(display("Could not load duration configration for {key:?} from repository"))]
    DurationConfig {
        key: StageState,
        source: GetDurationError,
    },
    #[snafu(display("Could not load notification configration for {key:?} from repository"))]
    NotificationConfig {
        key: StageState,
        source: GetNotificationError,
    },
}
