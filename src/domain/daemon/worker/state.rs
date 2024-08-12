use tokio::sync::oneshot::Sender;
use tokio::time::{Duration, Instant, Interval};

use crate::domain::daemon::worker::handle::{Command, QueryResponse};
use crate::domain::daemon::worker::routine::WorkerContext;
use crate::domain::entity::StageState;

#[derive(Debug)]
#[repr(transparent)]
pub struct WorkerState {
    inner: Option<WorkerStateInner>,
}

impl WorkerState {
    /// Creates a new [`WorkerState`].
    pub fn new() -> Self {
        Self {
            inner: Some(WorkerStateInner::new()),
        }
    }

    /// Do the business logic based on its inner state.
    pub async fn run(&mut self, context: &mut WorkerContext) {
        self.inner = match self.inner.take() {
            Some(inner) => Some(inner.run(context).await),
            None => unreachable!("`WorkerState`'s inner should not be `None`"),
        };
    }

    /// Returns `true` if is stopped of this [`WorkerState`].
    pub fn is_stopped(&self) -> bool {
        matches!(self.inner, Some(WorkerStateInner::Stopped(_)))
    }
}

#[enum_dispatch::enum_dispatch]
trait StateRun {
    async fn run(self, context: &mut WorkerContext) -> WorkerStateInner;
}

/// Actual implementation of running state of [`WorkerRoutine`].
#[derive(Debug)]
#[enum_dispatch::enum_dispatch(StateRun)]
enum WorkerStateInner {
    Ready(ReadyState),
    Running(RunningState),
    Paused(PausedState),
    Stopped(StoppedState),
}

impl WorkerStateInner {
    pub fn new() -> Self {
        Self::Ready(ReadyState)
    }
}

/// A state which indicates that the [`WorkerRoutine`] is ready to run.
#[derive(Debug)]
struct ReadyState;

impl StateRun for ReadyState {
    async fn run(self, context: &mut WorkerContext) -> WorkerStateInner {
        let stage = StageState::initial();
        let duration = *context.config.duration(stage).inner();
        let (start, timer) = spawn_timer(duration).await;

        RunningState {
            start,
            past: Duration::from_secs(0),
            timer,
            stage,
        }
        .into()
    }
}

/// A state which indicates that the [`WorkerRoutine`] is running, with a timer working
/// internally.
#[derive(Debug)]
struct RunningState {
    start: Instant,
    past: Duration,
    timer: Interval,
    stage: StageState,
}

impl StateRun for RunningState {
    async fn run(mut self, context: &mut WorkerContext) -> WorkerStateInner {
        tokio::select! {
            _ = self.timer.tick() => self.handle_tick(context).await,
            Some(command) = context.commands.recv() => match command {
                Command::Pause => self.handle_pause(),
                Command::Resume => self.handle_resume(),
                Command::Skip => self.handle_skip(context).await,
                Command::Query { responder } => self.handle_query(context, responder),
                Command::Stop => self.handle_stop(),
            },
            else => self.into(),
        }
    }
}

impl RunningState {
    async fn handle_tick(self, context: &mut WorkerContext) -> WorkerStateInner {
        let notification = context.config.notification(self.stage);

        if let Err(err) = context.notifier.notify(notification).await {
            tracing::error!(err = %err);
        }

        let stage = self.stage.next();
        let duration = *context.config.duration(stage).inner();
        let (start, timer) = spawn_timer(duration).await;

        RunningState {
            start,
            past: Duration::from_secs(0),
            timer,
            stage,
        }
        .into()
    }

    fn handle_resume(self) -> WorkerStateInner {
        self.into()
    }

    fn handle_pause(self) -> WorkerStateInner {
        PausedState {
            past: self.past + (Instant::now() - self.start),
            stage: self.stage,
        }
        .into()
    }

    async fn handle_skip(self, context: &mut WorkerContext) -> WorkerStateInner {
        let stage = self.stage.next();
        let duration = *context.config.duration(stage).inner();
        let (start, timer) = spawn_timer(duration).await;

        RunningState {
            start,
            past: Duration::from_secs(0),
            timer,
            stage,
        }
        .into()
    }

    fn handle_query(
        self,
        context: &mut WorkerContext,
        responder: Sender<QueryResponse>,
    ) -> WorkerStateInner {
        let _ = responder.send(QueryResponse {
            total: *context.config.duration(self.stage).inner(),
            past: self.past + (Instant::now() - self.start),
            stage: self.stage,
        });

        self.into()
    }

    fn handle_stop(self) -> WorkerStateInner {
        StoppedState.into()
    }
}

/// A state which indicates that the [`WorkerRoutine`] is paused. The time duration
/// goes by in this stage is stored for future resuming.
#[derive(Debug)]
struct PausedState {
    past: Duration,
    stage: StageState,
}

impl StateRun for PausedState {
    async fn run(self, context: &mut WorkerContext) -> WorkerStateInner {
        match context.commands.recv().await {
            Some(Command::Pause) => self.handle_pause(),
            Some(Command::Resume) => self.handle_resume(context).await,
            Some(Command::Skip) => self.handle_skip(context).await,
            Some(Command::Query { responder }) => self.handle_query(context, responder),
            Some(Command::Stop) => self.handle_stop(),
            None => self.into(),
        }
    }
}

impl PausedState {
    fn handle_pause(self) -> WorkerStateInner {
        self.into()
    }

    async fn handle_resume(self, context: &mut WorkerContext) -> WorkerStateInner {
        let duration = *context.config.duration(self.stage).inner();
        let (start, timer) = spawn_timer(duration - self.past).await;
        RunningState {
            start,
            past: self.past,
            timer,
            stage: self.stage,
        }
        .into()
    }

    async fn handle_skip(self, context: &mut WorkerContext) -> WorkerStateInner {
        let stage = self.stage.next();
        let duration = *context.config.duration(stage).inner();
        let (start, timer) = spawn_timer(duration).await;
        RunningState {
            start,
            past: Duration::from_secs(0),
            timer,
            stage,
        }
        .into()
    }

    fn handle_query(
        self,
        context: &mut WorkerContext,
        responder: Sender<QueryResponse>,
    ) -> WorkerStateInner {
        let _ = responder.send(QueryResponse {
            total: *context.config.duration(self.stage).inner(),
            past: self.past,
            stage: self.stage,
        });
        self.into()
    }

    fn handle_stop(self) -> WorkerStateInner {
        StoppedState.into()
    }
}

/// A state which indicates that [`WorkerRoutine`] should stop running.
#[derive(Debug)]
struct StoppedState;

impl StateRun for StoppedState {
    async fn run(self, _context: &mut WorkerContext) -> WorkerStateInner {
        self.into()
    }
}

async fn spawn_timer(duration: Duration) -> (Instant, Interval) {
    let mut timer = tokio::time::interval(duration);
    let start = timer.tick().await;
    (start, timer)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    use tokio::sync::mpsc::Sender;

    use crate::domain::daemon::outbound::{NotifyError, NotifyPort, NotifyRequest};
    use crate::domain::daemon::worker::routine::WorkerConfig;
    use crate::domain::entity::{NotificationMessage, StageDuration};

    #[tokio::test(start_paused = true)]
    async fn timer_operation() {
        let duration = Duration::from_secs(3);
        let (start, mut timer) = spawn_timer(duration.clone()).await;
        let now = timer.tick().await;
        assert_eq!(now - start, duration);
    }

    #[tokio::test(start_paused = true)]
    async fn ready_state_run() {
        let (_, mut context, _) = new_worker_context();
        let now = Instant::now();
        let state = ReadyState;
        let state = state.run(&mut context).await;

        match state {
            WorkerStateInner::Running(state) => {
                assert_eq!(state.start, now);
                assert_eq!(state.past, Duration::from_secs(0));
                assert_eq!(state.stage, StageState::Preparation);
            }
            _ => unreachable!(),
        }
    }

    #[tokio::test(start_paused = true)]
    async fn running_state_handle_tick() {
        let (_, mut context, notifier) = new_worker_context();
        let (start, state) = new_running_state().await;
        let state = state.handle_tick(&mut context).await;

        match state {
            WorkerStateInner::Running(state) => {
                assert_eq!(state.start, start);
                assert_eq!(state.past, Duration::from_secs(0));
                assert_eq!(state.stage, StageState::Concentration);
            }
            _ => unreachable!(),
        }

        let request = notifier.lock().unwrap().first().unwrap().clone();
        assert_eq!(request.summary, "Preparation");
        assert_eq!(request.body, None);
    }

    #[tokio::test(start_paused = true)]
    async fn running_state_handle_pause() {
        let (_, _, notifier) = new_worker_context();
        let (_, state) = new_running_state().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
        let state = state.handle_pause();

        match state {
            WorkerStateInner::Paused(state) => {
                assert_eq!(state.past, Duration::from_secs(1));
                assert_eq!(state.stage, StageState::Preparation);
            }
            _ => unreachable!(),
        }

        assert!(notifier.lock().unwrap().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn running_state_handle_skip() {
        let (_, mut context, notifier) = new_worker_context();
        let (start, state) = new_running_state().await;
        let state = state.handle_skip(&mut context).await;

        match state {
            WorkerStateInner::Running(state) => {
                assert_eq!(state.start, start);
                assert_eq!(state.past, Duration::from_secs(0));
                assert_eq!(state.stage, StageState::Concentration);
            }
            _ => unreachable!(),
        }

        assert!(notifier.lock().unwrap().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn paused_state_handle_resume() {
        let (_, mut context, notifier) = new_worker_context();
        let (start, state) = new_paused_state().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
        let state = state.handle_resume(&mut context).await;

        match state {
            WorkerStateInner::Running(state) => {
                assert_eq!(state.start, start + Duration::from_secs(1));
                assert_eq!(state.past, Duration::from_secs(0));
                assert_eq!(state.stage, StageState::Preparation);
            }
            _ => unreachable!(),
        }

        assert!(notifier.lock().unwrap().is_empty());
    }

    #[tokio::test(start_paused = true)]
    async fn paused_state_handle_skip() {
        let (_, mut context, notifier) = new_worker_context();
        let (start, state) = new_paused_state().await;
        let state = state.handle_skip(&mut context).await;

        match state {
            WorkerStateInner::Running(state) => {
                assert_eq!(state.start, start);
                assert_eq!(state.past, Duration::from_secs(0));
                assert_eq!(state.stage, StageState::Concentration);
            }
            _ => unreachable!(),
        }

        assert!(notifier.lock().unwrap().is_empty());
    }

    struct MockNotifier {
        notifications: Arc<Mutex<Vec<NotifyRequest>>>,
    }

    impl MockNotifier {
        fn new() -> (Arc<dyn NotifyPort>, Arc<Mutex<Vec<NotifyRequest>>>) {
            let notifier = Arc::new(Mutex::new(Vec::new()));
            let res = Self {
                notifications: Arc::clone(&notifier),
            };
            (Arc::new(res), notifier)
        }
    }

    #[async_trait::async_trait]
    impl NotifyPort for MockNotifier {
        async fn notify_impl(&self, request: NotifyRequest) -> Result<(), NotifyError> {
            self.notifications.lock().unwrap().push(request);
            Ok(())
        }
    }

    fn new_worker_context() -> (
        Sender<Command>,
        WorkerContext,
        Arc<Mutex<Vec<NotifyRequest>>>,
    ) {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        let new_duration = |d| StageDuration::try_new(d).unwrap();
        let new_message = |s: &str| NotificationMessage::try_new(s.to_owned(), None).unwrap();
        let (mock, data) = MockNotifier::new();

        let context = WorkerContext {
            config: WorkerConfig {
                preparation_duration: new_duration(5),
                concentration_duration: new_duration(20),
                relaxation_duration: new_duration(10),
                preparation_notification: new_message("Preparation"),
                concentration_notification: new_message("Concentration"),
                relaxation_notification: new_message("Relaxation"),
            },
            commands: receiver,
            notifier: mock,
        };

        (sender, context, data)
    }

    async fn new_running_state() -> (Instant, RunningState) {
        let (start, timer) = spawn_timer(Duration::from_secs(5)).await;
        let state = RunningState {
            start,
            past: Duration::from_secs(0),
            timer,
            stage: StageState::Preparation,
        };
        (start, state)
    }

    async fn new_paused_state() -> (Instant, PausedState) {
        let state = PausedState {
            past: Duration::from_secs(0),
            stage: StageState::Preparation,
        };
        (Instant::now(), state)
    }
}
