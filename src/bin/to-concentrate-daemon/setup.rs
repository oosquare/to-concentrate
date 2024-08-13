use std::path::Path;
use std::sync::Arc;

use snafu::{prelude::*, Whatever};
use to_concentrate::daemon::config::Configuration;
use to_concentrate::daemon::outbound::NotifyService;
use to_concentrate::daemon::repository::{DurationConfiguration, NotificationConfiguration};
use to_concentrate::daemon::{ProcessController, Server};
use to_concentrate::domain::daemon::ApplicationCore;
use to_concentrate::utils::xdg::{Xdg, XdgBaseKind};
use tokio::net::UnixListener;

use crate::cli::Arguments;

const APP_NAME: &str = "to-concentrate";

pub async fn bootstrap(arg: Arguments) -> Result<Server, Whatever> {
    process(&arg)?;

    logger()?;
    let configuration = configuration(&arg);
    let listener = listener()?;
    let core = core(configuration).await?;

    let server = Server::new(listener, core);
    Ok(server)
}

fn process(arg: &Arguments) -> Result<(), Whatever> {
    ProcessController::new(APP_NAME.to_owned(), arg.daemonize)
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

fn configuration(arg: &Arguments) -> Arc<Configuration> {
    let configuration = match &arg.config {
        Some(path) => Configuration::with_path(path.clone()),
        None => Configuration::with_xdg(APP_NAME.to_owned()),
    };

    Arc::new(configuration)
}

fn listener() -> Result<UnixListener, Whatever> {
    let path = Xdg::new(Path::new(APP_NAME))
        .and_then(|xdg| xdg.resolve(XdgBaseKind::Runtime, "daemon.socket"))
        .whatever_context("Could not resolve XDG runtime directory")?;

    let socket = UnixListener::bind(&path)
        .whatever_context(format!("Could not bind to {}", path.display()))?;

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
