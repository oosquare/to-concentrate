use std::path::PathBuf;
use std::process::Stdio;

use sysinfo::System;
use tokio::process::Command;
use tracing::Level;

use crate::daemon::runtime::{ControlProcessError, ProcessController};
use crate::domain::client::outbound::{InitDaemonError, InitPort};

#[derive(Debug)]
pub struct InitService {
    executable: Option<PathBuf>,
    pid_file: PathBuf,
    daemon_name: String,
    config: Option<PathBuf>,
    verbosity: Level,
}

impl InitService {
    pub fn new(
        executable: Option<PathBuf>,
        pid_file: PathBuf,
        daemon_name: String,
        config: Option<PathBuf>,
        verbosity: Level,
    ) -> Self {
        Self {
            executable,
            pid_file,
            daemon_name,
            config,
            verbosity,
        }
    }

    fn detect_instance(&self) -> Result<(), InitDaemonError> {
        let system = System::new_all();
        match ProcessController::detect_instance(&system, &self.pid_file, &self.daemon_name) {
            Ok(()) => Ok(()),
            Err(ControlProcessError::MultipleProcesses) => Err(InitDaemonError::AlreadyRunning),
            Err(err) => Err(InitDaemonError::Unknown {
                message: "Could not detect daemon".to_owned(),
                source: Some(err.into()),
            }),
        }
    }
}

#[async_trait::async_trait]
impl InitPort for InitService {
    async fn init(&self) -> Result<(), InitDaemonError> {
        self.detect_instance()?;

        let mut command = match &self.executable {
            Some(executable) => Command::new(executable),
            None => Command::new(&self.daemon_name),
        };

        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        command.arg("--verbosity").arg(self.verbosity.to_string());
        command.arg("--daemonize");

        if let Some(path) = self.config.as_ref() {
            command.arg("--config").arg(path);
        }

        let mut child = command.spawn().map_err(|err| InitDaemonError::Unknown {
            message: "Could not spawn daemon process".to_owned(),
            source: Some(err.into()),
        })?;

        let status = child.wait().await.map_err(|err| InitDaemonError::Unknown {
            message: "Could not get daemon status".to_owned(),
            source: Some(err.into()),
        })?;

        if !status.success() {
            Err(InitDaemonError::Unknown {
                message: "Daemon exited abnormally".to_owned(),
                source: None,
            })
        } else {
            Ok(())
        }
    }
}
