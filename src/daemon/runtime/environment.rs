use std::fs;
use std::io::Error as IoError;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use snafu::prelude::*;

/// Helper for setting up the daemon's running environment.
#[derive(Debug, Default)]
pub struct Environment {
    directories: Vec<PathBuf>,
    permissions: Vec<(PathBuf, u32)>,
}

impl Environment {
    /// Creates a new [`Environment`].
    pub fn new() -> Self {
        Self {
            directories: Vec::new(),
            permissions: Vec::new(),
        }
    }

    /// Register a path that needs to be created if it doesn't exist.
    pub fn register_directory<P: AsRef<Path>>(&mut self, directory: P) {
        self.directories.push(directory.as_ref().to_path_buf());
    }

    /// Register a path that needs to be set with the given permission.
    pub fn register_permission<P: AsRef<Path>>(&mut self, path: P, permission: u32) {
        self.permissions
            .push((path.as_ref().to_path_buf(), permission))
    }

    /// Setup the environment.
    ///
    /// # Errors
    ///
    /// This function will return an error if any system error occurs.
    pub fn setup(self) -> Result<(), SetupEnvironmentError> {
        Self::setup_directories(&self.directories)?;
        Self::setup_permissions(&self.permissions)?;
        Ok(())
    }

    /// Do directories creation.
    ///
    /// # Errors
    ///
    /// This function will return an error if any system error occurs.
    fn setup_directories(directories: &Vec<PathBuf>) -> Result<(), SetupEnvironmentError> {
        for dir in directories {
            fs::create_dir_all(dir).context(CreateDirectorySnafu { dir })?;
        }

        Ok(())
    }

    /// Do permission modification.
    ///
    /// # Errors
    ///
    /// This function will return an error if any system error occurs.
    fn setup_permissions(permissions: &Vec<(PathBuf, u32)>) -> Result<(), SetupEnvironmentError> {
        for (path, permission) in permissions {
            let metadata = fs::metadata(path).context(SetPermissionSnafu {
                path: path.as_path(),
                permission: *permission,
            })?;

            if metadata.permissions().mode() != *permission {
                metadata.permissions().set_mode(*permission);
            }
        }

        Ok(())
    }
}

/// An error for setting up the running environment.
#[derive(Debug, Snafu, Clone)]
pub enum SetupEnvironmentError {
    #[snafu(display("Could not create directory {}", dir.display()))]
    CreateDirectory {
        dir: PathBuf,
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
    #[snafu(display("Could not set {}'s permission to {permission}", path.display()))]
    SetPermission {
        path: PathBuf,
        permission: u32,
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::{prelude::*, TempDir};
    use predicates::path as path_pred;

    #[test]
    fn environment_setup_directories() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        let dir = tmp.child("dir");
        let subdir = tmp.child("dir/subdir");
        dir.assert(path_pred::missing());
        subdir.assert(path_pred::missing());

        let mut env = Environment::new();
        env.register_directory(subdir.as_ref());
        env.setup().unwrap();

        dir.assert(path_pred::is_dir());
        subdir.assert(path_pred::is_dir());
    }

    #[test]
    fn environment_setup_permission() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        tmp.child("file").touch().unwrap();
        let file = tmp.child("file").to_path_buf();
        println!("{}", file.display());

        let mut env = Environment::new();
        env.register_permission(&file, 0o644);
        env.setup().unwrap();

        let perm = fs::metadata(&file).unwrap().permissions().mode();
        assert_eq!(perm & ((1 << 9) - 1), 0o644);
    }
}
