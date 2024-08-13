use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use snafu::prelude::*;
use xdg::{BaseDirectories, BaseDirectoriesError};

/// Helper for using XDG base directories.
pub struct Xdg {
    base: BaseDirectories,
}

impl Xdg {
    /// Create a [`Xdg`]. All subsequent file system operations in XDG base
    /// directories will be performed in a subdirectory named prefix.
    ///
    /// # Errors
    ///
    /// This function will return an error if XDG settings is missing.
    pub fn new<P: AsRef<Path>>(prefix: P) -> Result<Self, XdgError> {
        let base = BaseDirectories::with_prefix(prefix).context(InitSnafu)?;
        Ok(Self { base })
    }

    /// Resolve the absolute path for the file.
    ///
    /// # Errors
    ///
    /// This function will return an error if XDG runtime directory is not
    /// available when resolving it.
    pub fn resolve<P: AsRef<Path>>(&self, kind: XdgBaseKind, file: P) -> Result<PathBuf, XdgError> {
        match kind {
            XdgBaseKind::Config => Ok(self.base.get_config_file(file)),
            XdgBaseKind::Runtime => self.base.get_runtime_file(file).context(FileSystemSnafu {
                message: "XDG runtime directory is not available",
            }),
        }
    }

    /// Resolve the absolute path for the file and create the leading
    /// directories if they didn't exist before.
    ///
    /// # Errors
    ///
    /// This function will return an error if creating directories fails.
    pub fn resolve_create<P: AsRef<Path>>(
        &self,
        kind: XdgBaseKind,
        file: P,
    ) -> Result<PathBuf, XdgError> {
        let res = match kind {
            XdgBaseKind::Config => self.base.place_config_file(file),
            XdgBaseKind::Runtime => self.base.place_runtime_file(file),
        };

        let path = res.context(FileSystemSnafu {
            message: "Could not create {kind} dirctory for application",
        })?;

        Ok(path)
    }
}

/// Kind of XDG base directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XdgBaseKind {
    Config,
    Runtime,
}

impl Display for XdgBaseKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Config => f.write_str("configuration"),
            Self::Runtime => f.write_str("runtime"),
        }
    }
}

/// An error for XDG-related operations.
#[derive(Debug, Snafu, Clone)]
pub enum XdgError {
    #[snafu(display("Could not get XDG settings"))]
    Init {
        #[snafu(source(from(BaseDirectoriesError, Arc::new)))]
        source: Arc<BaseDirectoriesError>,
    },
    #[snafu(display("File system error: {message}"))]
    FileSystem {
        message: String,
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
}
