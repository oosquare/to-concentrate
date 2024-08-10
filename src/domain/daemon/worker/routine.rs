use std::sync::Arc;

use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use crate::domain::daemon::outbound::NotifyPort;
use crate::domain::daemon::worker::handle::Command;
use crate::domain::daemon::worker::state::WorkerState;
use crate::domain::entity::{NotificationMessage, StageDuration, StageState};

/// A type that stores configurations required by [`WorkerRoutine`]
/// initialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerConfig {
    pub preparation_duration: StageDuration,
    pub concentration_duration: StageDuration,
    pub relaxation_duration: StageDuration,
    pub preparation_notification: NotificationMessage,
    pub concentration_notification: NotificationMessage,
    pub relaxation_notification: NotificationMessage,
}

impl WorkerConfig {
    /// Get the duration corresponding to stage.
    pub fn duration(&self, stage: StageState) -> &StageDuration {
        match stage {
            StageState::Preparation => &self.preparation_duration,
            StageState::Concentration => &self.concentration_duration,
            StageState::Relaxation => &self.relaxation_duration,
        }
    }

    /// Get the notification message corresponding to stage.
    pub fn notification(&self, stage: StageState) -> &NotificationMessage {
        match stage {
            StageState::Preparation => &self.preparation_notification,
            StageState::Concentration => &self.concentration_notification,
            StageState::Relaxation => &self.relaxation_notification,
        }
    }
}

/// A [`WorkerContext`] stores all objects relavent to the [`WorkerRoutine`]
/// and the business logic.
pub struct WorkerContext {
    pub config: WorkerConfig,
    pub commands: Receiver<Command>,
    pub notifier: Arc<dyn NotifyPort>,
}

/// A type responsible for the daemon's main business logic. A [`WorkerRoutine`]
/// runs on background, receiving [`Command`]s from [`WorkerHandle`].
pub struct WorkerRoutine {
    context: WorkerContext,
    state: WorkerState,
}

impl WorkerRoutine {
    /// Spawn a running [`WorkerRoutine`] on background.
    pub fn spawn(
        config: WorkerConfig,
        commands: Receiver<Command>,
        notifier: Arc<dyn NotifyPort>,
    ) -> JoinHandle<()> {
        tokio::spawn(async {
            let mut worker = Self {
                context: WorkerContext {
                    config,
                    commands,
                    notifier,
                },
                state: WorkerState::new(),
            };
            worker.run().await;
        })
    }

    /// Main part of its business logic.
    async fn run(&mut self) {
        while !self.state.is_stopped() {
            self.state.run(&mut self.context).await;
        }
    }
}
