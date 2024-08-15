use std::fs::File;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use daemonize::{Daemonize, Error as DaemonizeError};
use snafu::prelude::*;
use sysinfo::{Pid, System};

/// A process manager responsible for daemonization and preventing multiple
/// running instance.
#[derive(Debug)]
pub struct ProcessController {
    app_name: String,
    pid_file: PathBuf,
    daemonize: bool,
}

impl ProcessController {
    /// Creates a new [`ProcessController`].
    pub fn new(app_name: String, pid_file: PathBuf, daemonize: bool) -> Self {
        Self {
            app_name,
            pid_file,
            daemonize,
        }
    }

    /// Finish process-related work, such as daemonization and multiple instance
    /// detection.
    ///
    /// # Errors
    ///
    /// This function will return an error if the preapration fails.
    pub fn start(self) -> Result<(), ControlProcessError> {
        let system = System::new_all();
        Self::detect_instance(&system, &self.pid_file, &self.app_name)?;

        if self.daemonize {
            Daemonize::new()
                .pid_file(&self.pid_file)
                .start()
                .context(DaemonizeSnafu)?;
        } else {
            let pid =
                sysinfo::get_current_pid().map_err(|err| GetPidSnafu { message: err }.build())?;
            Self::write_pid(&self.pid_file, pid)?;
        }

        Ok(())
    }

    pub fn detect_instance<P: AsRef<Path>>(
        system: &System,
        pid_file: P,
        app_name: &str,
    ) -> Result<(), ControlProcessError> {
        let mut file = match File::open(pid_file) {
            Ok(file) => file,
            Err(err) => match err.kind() {
                IoErrorKind::NotFound => return Ok(()),
                _ => {
                    return Err(err).context(FileSystemSnafu {
                        message: "Could not open PID file",
                    })
                }
            },
        };

        let mut content = String::new();
        file.read_to_string(&mut content).context(FileSystemSnafu {
            message: "Could not open PID file",
        })?;

        let pid = content
            .trim()
            .parse::<Pid>()
            .map_err(|_| InvalidPidFileSnafu.build())?;

        if let Some(proc) = system.process(pid) {
            let name = proc.name().to_string_lossy();
            if name.contains(app_name) {
                MultipleProcessesSnafu.fail()
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn write_pid<P: AsRef<Path>>(pid_file: P, pid: Pid) -> Result<(), ControlProcessError> {
        let mut file = File::create(pid_file).context(FileSystemSnafu {
            message: "Could not write PID",
        })?;
        file.write_all(pid.to_string().as_bytes())
            .context(FileSystemSnafu {
                message: "Could not write PID",
            })?;
        Ok(())
    }
}

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum ControlProcessError {
    #[snafu(display("File system error: {message}"))]
    FileSystem {
        message: String,
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
    #[snafu(display("Could not start multiple daemon processes"))]
    MultipleProcesses,
    #[snafu(display("Could not ensure process uniqueness with invalid PID file"))]
    InvalidPidFile,
    #[snafu(display("Failed to get PID: {message}"))]
    GetPid { message: String },
    #[snafu(display("Could not daemonize the process"))]
    Daemonize { source: DaemonizeError },
}
