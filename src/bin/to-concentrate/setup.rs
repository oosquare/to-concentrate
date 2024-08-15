use std::cell::LazyCell;
use std::path::PathBuf;
use std::sync::Arc;

use snafu::{prelude::*, Whatever};
use to_concentrate::client::app::connector::{Connector, UnixConnector};
use to_concentrate::client::config;
use to_concentrate::client::outbound::{
    InitService, PauseService, QueryService, ResumeService, SkipService,
};
use to_concentrate::client::Client;
use to_concentrate::domain::client::ApplicationCore;
use to_concentrate::utils::xdg::{Xdg, XdgBaseKind};
use tracing::Level;

use crate::cli::{Arguments, Command};

const APP_NAME: &str = "to-concentrate";
const DAEMON_NAME: &str = "to-concentrate-daemon";

struct EnvironmentPath {
    socket: PathBuf,
    pid: PathBuf,
}

pub fn bootstrap(args: &Arguments) -> Result<Client, Whatever> {
    let env_path = environment(args)?;
    let core = core(args, env_path);
    let client = Client::new(core);
    Ok(client)
}

fn environment(args: &Arguments) -> Result<EnvironmentPath, Whatever> {
    let res = match &args.config {
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
    Ok(env_path)
}

fn core(args: &Arguments, env_path: EnvironmentPath) -> Arc<ApplicationCore> {
    let executable = match &args.command {
        Command::Init { executable, .. } => executable.clone(),
        _ => None,
    };

    let verbosity = match &args.command {
        Command::Init { verbosity, .. } => *verbosity,
        _ => Level::INFO,
    };

    let connector: Arc<dyn Connector> = Arc::new(UnixConnector::new(env_path.socket));

    let init_port = Arc::new(InitService::new(
        executable,
        env_path.pid.to_path_buf(),
        DAEMON_NAME.to_owned(),
        args.config.clone(),
        verbosity,
    ));

    let pause_port = Arc::new(PauseService::new(Arc::clone(&connector)));
    let resume_port = Arc::new(ResumeService::new(Arc::clone(&connector)));
    let query_port = Arc::new(QueryService::new(Arc::clone(&connector)));
    let skip_port = Arc::new(SkipService::new(Arc::clone(&connector)));

    let core = ApplicationCore::setup(init_port, pause_port, resume_port, query_port, skip_port);
    Arc::new(core)
}
