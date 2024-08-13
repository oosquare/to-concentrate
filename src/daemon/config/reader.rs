use std::fs::File;
use std::io::{Error as IoError, ErrorKind, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use snafu::prelude::*;

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

# The `runtime` section specifies the paths to some runtime files. Leave
# them empty to use default settings. Currently environment variables is not
# supported.
# [runtime]
# socket = "/path/to/unix/socket"
# runtime = "/path/to/pid/file"
"#;

/// A reader which reads the configuration content and creates a default
/// configuration file if it is missing.
pub struct ContentReader {
    path: PathBuf,
    create_new: bool,
}

impl ContentReader {
    /// Creates a new [`ContentReader`].
    pub fn new<P: AsRef<Path>>(path: P, create_new: bool) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            create_new,
        }
    }

    /// Read content from the file.
    ///
    /// # Errors
    ///
    /// This function will return an error if file doesn't exist or it fails to
    /// create a configuration file.
    pub fn read(self) -> Result<String, ReadContentError> {
        let Self { path, create_new } = self;
        let mut file = Self::open_configuration(path, create_new)?;
        let mut content = String::new();
        file.read_to_string(&mut content).context(FileSystemSnafu {
            when: "Reading configuration",
        })?;
        Ok(content)
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
                _ => Err(err).context(FileSystemSnafu {
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

        let reader = ContentReader::new(file.to_path_buf(), false);
        assert_eq!(reader.read().unwrap(), content);
    }

    #[test]
    fn open_configuration_not_found() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        let file = tmp.child("config.toml");
        file.assert(path_pred::missing());
        assert!(matches!(
            ContentReader::open_configuration(file.to_path_buf(), false),
            Err(ReadContentError::NotFound { .. })
        ));
    }

    #[test]
    fn create_configuration() {
        let tmp = TempDir::new().expect("Test environment should support temporary directories");
        let file = tmp.child("config.toml");
        file.assert(path_pred::missing());
        assert!(ContentReader::open_configuration(file.to_path_buf(), true).is_ok());
        file.assert(DEFAULT_CONTENT);
    }
}
