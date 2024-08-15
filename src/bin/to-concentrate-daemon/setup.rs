use std::cell::LazyCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use snafu::{prelude::*, Whatever};
use to_concentrate::daemon::app::listener::Listener;
use to_concentrate::daemon::config::{self, Configuration};
use to_concentrate::daemon::outbound::NotifyService;
use to_concentrate::daemon::repository::{DurationConfiguration, NotificationConfiguration};
use to_concentrate::daemon::runtime::{Environment, ProcessController};
use to_concentrate::daemon::{Server, UnixListener};
use to_concentrate::domain::daemon::ApplicationCore;
use to_concentrate::tracing_report;
use to_concentrate::utils::xdg::{Xdg, XdgBaseKind};

use crate::cli::Arguments;

const APP_NAME: &str = "to-concentrate";

struct EnvironmentPath {
    socket: PathBuf,
    pid: PathBuf,
}

#[tracing::instrument(skip(arg))]
pub async fn bootstrap(arg: Arguments) -> Result<Server, Whatever> {
    let (configuration, env_path) = configuration(&arg)
        .inspect(|_| tracing::info!("Loaded configuration"))
        .inspect_err(|err| tracing_report!(err))?;

    environment(&env_path)
        .inspect(|_| tracing::info!("Initialized environment"))
        .inspect_err(|err| tracing_report!(err))?;

    process(&arg, env_path.pid)
        .inspect(|_| tracing::info!("Finished process-related operations"))
        .inspect_err(|err| tracing_report!(err))?;

    let listener = listener(env_path.socket)
        .inspect(|_| tracing::info!("Initialized socket"))
        .inspect_err(|err| tracing_report!(err))?;

    let core = core(configuration)
        .await
        .inspect(|_| tracing::info!("Initialized server core"))
        .inspect_err(|err| tracing_report!(err))?;

    let server = Server::new(listener, core);
    tracing::info!("Initialized application");
    Ok(server)
}

fn environment(env_path: &EnvironmentPath) -> Result<(), Whatever> {
    let socket_parent = env_path.socket.parent().whatever_context(format!(
        "Invalid socket path: {}",
        env_path.socket.display()
    ))?;

    let pid_parent = env_path
        .pid
        .parent()
        .whatever_context(format!("Invalid PID path: {}", env_path.pid.display()))?;

    let mut env = Environment::new();
    env.register_directory(socket_parent);
    env.register_directory(pid_parent);
    env.setup().whatever_context("Could not setup environment")
}

fn process<P: AsRef<Path>>(arg: &Arguments, pid_path: P) -> Result<(), Whatever> {
    ProcessController::new(
        APP_NAME.to_owned(),
        pid_path.as_ref().to_path_buf(),
        arg.daemonize,
    )
    .start()
    .whatever_context("Could not prepare process")
}

fn configuration(arg: &Arguments) -> Result<(Arc<Configuration>, EnvironmentPath), Whatever> {
    let res = match &arg.config {
        Some(path) => config::load_with_path(path.clone()),
        None => config::load_with_xdg(APP_NAME.to_owned()),
    };

    let configuration = res.whatever_context("Could not load configuration")?;

    let xdg = LazyCell::new(|| Xdg::new(APP_NAME));

    let socket = match &configuration.runtime.socket {
        Some(socket) => socket.clone(),
        None => xdg
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|xdg| xdg.resolve(XdgBaseKind::Runtime, "daemon.socket"))
            .whatever_context("Could not use XDG base directories")?,
    };

    let pid = match &configuration.runtime.pid {
        Some(socket) => socket.clone(),
        None => xdg
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|xdg| xdg.resolve(XdgBaseKind::Runtime, "daemon.pid"))
            .whatever_context("Could not use XDG base directories")?,
    };

    let env_path = EnvironmentPath { socket, pid };
    Ok((Arc::new(configuration), env_path))
}

fn listener<P: AsRef<Path>>(path: P) -> Result<Box<dyn Listener>, Whatever> {
    let _ = fs::remove_file(&path);
    UnixListener::new(&path)
        .map(|listener| -> Box<dyn Listener> { Box::new(listener) })
        .whatever_context(format!("Could not bind to {}", path.as_ref().display()))
}

async fn core(config: Arc<Configuration>) -> Result<ApplicationCore, Whatever> {
    let notify_port = Arc::new(NotifyService::new(APP_NAME.to_owned()));
    let duration_repository = Arc::new(DurationConfiguration::new(Arc::clone(&config)));
    let notification_repository = Arc::new(NotificationConfiguration::new(config));

    ApplicationCore::setup(notify_port, duration_repository, notification_repository)
        .await
        .whatever_context("Could not setup application core")
}
