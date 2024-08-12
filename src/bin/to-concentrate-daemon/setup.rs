use std::sync::Arc;

use daemonize::Daemonize;
use snafu::{prelude::*, Whatever};
use to_concentrate::daemon::config::Configuration;
use to_concentrate::daemon::outbound::NotifyService;
use to_concentrate::daemon::repository::{DurationConfiguration, NotificationConfiguration};
use to_concentrate::daemon::Server;
use to_concentrate::domain::daemon::Application;
use tokio::net::UnixListener;
use xdg::BaseDirectories;

use crate::cli::Arguments;

const APP_NAME: &str = "to-concentrate";

pub async fn bootstrap(arg: Arguments) -> Result<Server, Whatever> {
    daemonize(&arg)?;

    logger()?;
    let configuration = configuration(&arg);
    let listener = listener(&arg)?;
    let core = core(configuration).await?;

    let server = Server::new(listener, core);
    Ok(server)
}

fn daemonize(arg: &Arguments) -> Result<(), Whatever> {
    if arg.daemonize {
        Daemonize::new()
            .start()
            .whatever_context("Could not daemonize the process")?;
    }
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

fn listener(arg: &Arguments) -> Result<UnixListener, Whatever> {
    let socket_path = match &arg.socket {
        Some(path) => path.clone(),
        None => {
            let base = BaseDirectories::with_prefix(APP_NAME.to_owned())
                .whatever_context("Could not use XDG runtime directory")?;
            base.place_runtime_file(&format!("{APP_NAME}-daemon.socket"))
                .whatever_context("Could not create runtime directory")?
        }
    };

    let socket = UnixListener::bind(&socket_path)
        .whatever_context(format!("Could not bind to {}", socket_path.display()))?;

    Ok(socket)
}

async fn core(config: Arc<Configuration>) -> Result<Application, Whatever> {
    let notify_port = Arc::new(NotifyService::new(APP_NAME.to_owned()));
    let duration_repository = Arc::new(DurationConfiguration::new(Arc::clone(&config)));
    let notification_repository = Arc::new(NotificationConfiguration::new(config));

    let core = Application::new(notify_port, duration_repository, notification_repository)
        .await
        .whatever_context("Could not setup application core")?;

    Ok(core)
}
