use std::path::{Path, PathBuf};
use std::sync::Arc;

use snafu::{prelude::*, ResultExt, Whatever};
use to_concentrate::daemon::config::{self, Configuration};
use to_concentrate::daemon::outbound::NotifyService;
use to_concentrate::daemon::repository::{DurationConfiguration, NotificationConfiguration};
use to_concentrate::daemon::runtime::Environment;
use to_concentrate::daemon::{ProcessController, Server};
use to_concentrate::domain::daemon::ApplicationCore;
use to_concentrate::utils::xdg::{Xdg, XdgBaseKind};
use tokio::net::UnixListener;

use crate::cli::Arguments;

const APP_NAME: &str = "to-concentrate";

pub struct EnvironmentPath {
    socket: PathBuf,
    pid: PathBuf,
}

pub async fn bootstrap(arg: Arguments) -> Result<Server, Whatever> {
    let configuration = configuration(&arg)?;
    let env = environment()?;
    process(&arg, env.pid)?;

    logger()?;
    let listener = listener(env.socket)?;
    let core = core(configuration).await?;

    let server = Server::new(listener, core);
    Ok(server)
}

fn environment() -> Result<EnvironmentPath, Whatever> {
    let xdg = Xdg::new(APP_NAME).whatever_context("Could not use XDG base directories")?;
    let mut env = Environment::new();

    let socket_path = xdg
        .resolve(XdgBaseKind::Runtime, "daemon.socket")
        .whatever_context("Could not use XDG base directories")?;

    let socket_parent = socket_path
        .parent()
        .whatever_context(format!("Invalid socket path: {}", socket_path.display()))?;

    env.register_directory(socket_parent);
    env.register_permission(socket_parent, 0o700);

    let pid_path = xdg
        .resolve(XdgBaseKind::Runtime, "daemon.pid")
        .whatever_context("Could not use XDG base directories")?;

    let pid_parent = pid_path
        .parent()
        .whatever_context(format!("Invalid PID path: {}", pid_path.display()))?;

    env.register_directory(pid_parent);
    env.register_permission(pid_parent, 0o700);

    env.setup()
        .whatever_context("Could not setup environment")?;

    Ok(EnvironmentPath {
        socket: socket_path,
        pid: pid_path,
    })
}

fn process<P: AsRef<Path>>(arg: &Arguments, pid_path: P) -> Result<(), Whatever> {
    ProcessController::new(
        APP_NAME.to_owned(),
        pid_path.as_ref().to_path_buf(),
        arg.daemonize,
    )
    .start()
    .whatever_context("Could not prepare process")?;
    Ok(())
}

fn logger() -> Result<(), Whatever> {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)
        .whatever_context("Could not setup logger")?;
    Ok(())
}

fn configuration(arg: &Arguments) -> Result<Arc<Configuration>, Whatever> {
    let res = match &arg.config {
        Some(path) => config::load_with_path(path.clone()),
        None => config::load_with_xdg(APP_NAME.to_owned()),
    };

    let configuration = res.whatever_context("Could not load configuration")?;
    Ok(Arc::new(configuration))
}

fn listener<P: AsRef<Path>>(path: P) -> Result<UnixListener, Whatever> {
    let socket = UnixListener::bind(&path)
        .whatever_context(format!("Could not bind to {}", path.as_ref().display()))?;

    Ok(socket)
}

async fn core(config: Arc<Configuration>) -> Result<ApplicationCore, Whatever> {
    let notify_port = Arc::new(NotifyService::new(APP_NAME.to_owned()));
    let duration_repository = Arc::new(DurationConfiguration::new(Arc::clone(&config)));
    let notification_repository = Arc::new(NotificationConfiguration::new(config));

    let core = ApplicationCore::setup(notify_port, duration_repository, notification_repository)
        .await
        .whatever_context("Could not setup application core")?;

    Ok(core)
}
