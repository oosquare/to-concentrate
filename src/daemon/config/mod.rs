mod content;
mod reader;

use std::path::Path;

pub use content::Configuration;
pub use reader::ReadContentError;

use snafu::prelude::*;
use toml::de::Error as DeError;

use crate::utils::xdg::{Xdg, XdgBaseKind, XdgError};

use reader::ContentReader;

/// An error type for loading configuraton from files.
#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum LoadConfigurationError {
    #[snafu(display("Could not resolve XDG configuration directory"))]
    XdgConfig { source: XdgError },
    #[snafu(display("Could not read content from file"))]
    Read { source: ReadContentError },
    #[snafu(display("Could not parse invalid configurations"))]
    Parse { source: DeError },
}

/// Read configuration from given path. Optionally create one from default
/// template if it doesn't exists.
///
/// # Errors
///
/// This function will return an error if reading content from file fails or
/// parsing configuration fails.
pub fn load<P: AsRef<Path>>(
    path: P,
    create_new: bool,
) -> Result<Configuration, LoadConfigurationError> {
    let content = ContentReader::new(path.as_ref(), create_new)
        .read()
        .context(ReadSnafu)?;
    toml::from_str(&content).context(ParseSnafu)
}

/// Read configuration from a custom path. This won't create any new file by
/// default.
///
/// # Errors
///
/// This function will return an error if reading content from file fails or
/// parsing configuration fails.
pub fn load_with_path<P: AsRef<Path>>(path: P) -> Result<Configuration, LoadConfigurationError> {
    load(path, false)
}

/// Read configuration from XDG configuration directory. Create one from default
/// template if it doesn't exists.
///
/// # Errors
///
/// This function will return an error if reading content from file fails or
/// parsing configuration fails.
pub fn load_with_xdg(app_name: String) -> Result<Configuration, LoadConfigurationError> {
    let path = Xdg::new(Path::new(&app_name))
        .and_then(|xdg| xdg.resolve_create(XdgBaseKind::Config, "config.toml"))
        .context(XdgConfigSnafu)?;
    load(path, true)
}
