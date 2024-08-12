use std::fs::File;
use std::io::{Error as IoError, ErrorKind, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use snafu::{prelude::*, ResultExt};
use xdg::{BaseDirectories, BaseDirectoriesError};

pub const DEFAULT_CONTENT: &str = r#"
# This configuration file is generated automatically. Feel free to do some
# modification.

# The `duration` section specifies the duration of each stage in seconds.
[duration]
preparation = 900
concentration = 2400
relaxation = 600

# The `notification.<stage>` section specifies the message shown in desktop
# notifications. `body` is optional.
[notification.preparation]
summary = "Preparation Stage End"
body = "It's time to start concentrating on learning."

[notification.concentration]
summary = "Concentration Stage End"
body = "Well done! Remember to have a rest."

[notification.relaxation]
summary = "Relaxation Stage End"
body = "Feel energetic now? Let's continue."
"#;

type PathGetter = Box<dyn FnOnce() -> Result<PathBuf, ReadContentError> + Send>;

/// A lazy reader which reads the configuration content and creates a default
/// configuration file if it is missing.
pub struct LazyContentReader {
    path_getter: PathGetter,
    create_new: bool,
}

impl LazyContentReader {
    /// Creates a new [`LazyContentReader`].
    fn new(path_getter: PathGetter, create_new: bool) -> Self {
        Self {
            path_getter,
            create_new,
        }
    }

    /// Create a new [`LazyContentReader`] with this application's name.
    /// Afterwards, it'll read content from XDG config directory.
    pub fn with_xdg(app_name: String) -> Self {
        let path_getter = move || Self::resolve_configuration_path(app_name);
        Self::new(Box::new(path_getter), true)
    }

    /// Create a new [`LazyContentReader`] with a custom path. Automatic file
    /// creation is disabled.
    pub fn with_path(path: PathBuf) -> Self {
        let path_getter = move || Ok(path);
        Self::new(Box::new(path_getter), false)
    }

    /// Read content from the file.
    ///
    /// # Errors
    ///
    /// This function will return an error if file doesn't exist or it fails to
    /// create a configuration file.
    pub fn read(self) -> Result<String, ReadContentError> {
        let Self {
            path_getter,
            create_new,
        } = self;

        let mut file = Self::open_configuration(path_getter()?, create_new)?;
        let mut content = String::new();
        file.read_to_string(&mut content).context(FileSystemSnafu {
            when: "Reading configuration",
        })?;
        Ok(content)
    }

    /// Resolve an absolute path to configuration file and try to create its
    /// leading directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the resolution fails.
    fn resolve_configuration_path(app_name: String) -> Result<PathBuf, ReadContentError> {
        let prefix = PathBuf::from(app_name).to_path_buf();

        let path = BaseDirectories::with_prefix(prefix)
            .context(XdgConfigSnafu)?
            .place_config_file("config.toml")
            .context(FileSystemSnafu {
                when: "Creating XDG config directory",
            })?;

        Ok(path)
    }

    /// Open the configuration file. Create one if specified when it doesn't
    /// exists before.
    ///
    /// # Errors
    ///
    /// This function will return an error if file doesn't exists or it fails to
    /// create a default one.
    fn open_configuration(path: PathBuf, create_new: bool) -> Result<File, ReadContentError> {
        match File::open(path.as_path()) {
            Ok(file) => Ok(file),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {
                    if create_new {
                        Self::create_configuration(path.as_path())
                    } else {
                        NotFoundSnafu { path }.fail()
                    }
                }
                _ => Err(err.into()).context(FileSystemSnafu {
                    when: "Opening configuration file",
                }),
            },
        }
    }

    /// Create a default configuration file.
    ///
    /// # Errors
    ///
    /// This function will return an error if the creation fails.
    fn create_configuration<P: AsRef<Path>>(path: P) -> Result<File, ReadContentError> {
        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .context(FileSystemSnafu {
                when: "Creating configuration file",
            })?;

        file.write_all(DEFAULT_CONTENT.as_bytes())
            .context(FileSystemSnafu {
                when: "Writing default configuration content",
            })?;

        file.seek(std::io::SeekFrom::Start(0))
            .context(FileSystemSnafu {
                when: "Reseting file cursor position to start",
            })?;

        Ok(file)
    }
}

/// An error type for reading content from the configuration file.
#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum ReadContentError {
    #[snafu(display("Could not open inexistent file {}", path.display()))]
    NotFound { path: PathBuf },
    #[snafu(display("Could not resolve XDG config directory"))]
    XdgConfig {
        #[snafu(source(from(BaseDirectoriesError, Arc::new)))]
        source: Arc<BaseDirectoriesError>,
    },
    #[snafu(display("Could not create default configuration: {when}"))]
    FileSystem {
        when: String,
        #[snafu(source(from(IoError, Arc::new)))]
        source: Arc<IoError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use predicates::path as path_pred;

    #[test]
    fn read_configuration() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        let file = tmp.child("config.toml");
        let content = "content for testing";
        file.write_str(&content).unwrap();

        let reader = LazyContentReader::with_path(file.to_path_buf());
        assert_eq!(reader.read().unwrap(), content);
    }

    #[test]
    fn open_configuration_not_found() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        let file = tmp.child("config.toml");
        file.assert(path_pred::missing());
        assert!(matches!(
            LazyContentReader::open_configuration(file.to_path_buf(), false),
            Err(ReadContentError::NotFound { .. })
        ));
    }

    #[test]
    fn create_configuration() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        let file = tmp.child("config.toml");
        file.assert(path_pred::missing());
        assert!(LazyContentReader::open_configuration(file.to_path_buf(), true).is_ok());
        file.assert(DEFAULT_CONTENT);
    }
}