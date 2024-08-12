mod content;
mod reader;

pub use content::ConfigurationContent;
pub use reader::ReadContentError;

use std::path::PathBuf;
use std::sync::LazyLock;

use snafu::prelude::*;
use toml::de::Error as DeError;

use reader::LazyContentReader;

type LazyConfiguration = Result<ConfigurationContent, LoadConfigurationError>;
type LazyConfigurationLoader = Box<dyn FnOnce() -> LazyConfiguration + Send>;
type LazyRefConfiguration<'a> = Result<&'a ConfigurationContent, LoadConfigurationError>;

/// In-memory configuration storage with lazy loading.
pub struct Configuration {
    content: LazyLock<LazyConfiguration, LazyConfigurationLoader>,
}

impl Configuration {
    /// Creates a new [`Configuration`].
    fn new(reader: LazyContentReader) -> Self {
        let loader = move || {
            let content = reader.read().context(ReadSnafu)?;
            let config = toml::from_str(&content).context(ParseSnafu)?;
            Ok(config)
        };
        let loader: LazyConfigurationLoader = Box::new(loader);
        let content = LazyLock::new(loader);
        Self { content }
    }

    /// Creates a new [`Configuration`] with this application's name.
    /// Afterwards, it'll load configurations from XDG config directory.
    pub fn with_xdg(app_name: String) -> Self {
        Self::new(LazyContentReader::with_xdg(app_name))
    }

    /// Create a new [`Configuration`] with a custom directory. It will fail
    /// on providing configurations if the file is not found.
    pub fn with_custom_dir(custom_dir: PathBuf) -> Self {
        Self::new(LazyContentReader::with_custom_dir(custom_dir))
    }

    /// Return the configuration. Load it if it hasn't been loaded into memory.
    pub fn get(&self) -> LazyRefConfiguration {
        self.content.as_ref().map_err(Clone::clone)
    }
}

/// An error type for loading configuraton from files.
#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum LoadConfigurationError {
    #[snafu(display("Could not read content from file"))]
    Read { source: ReadContentError },
    #[snafu(display("Could not parse invalid configurations"))]
    Parse { source: DeError },
}
