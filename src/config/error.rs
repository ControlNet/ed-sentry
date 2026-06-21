use std::fmt;
use std::path::PathBuf;

use super::ConfigSource;

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        config_source: ConfigSource,
        source: std::io::Error,
    },
    MalformedToml {
        path: Option<PathBuf>,
        config_source: ConfigSource,
        source: Box<toml::de::Error>,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source, .. } => {
                write!(
                    formatter,
                    "failed to read config {}: {source}",
                    path.display()
                )
            }
            Self::MalformedToml { path, source, .. } => match path {
                Some(path) => write!(
                    formatter,
                    "malformed TOML config {}: {source}",
                    path.display()
                ),
                None => write!(formatter, "malformed TOML config: {source}"),
            },
        }
    }
}

impl std::error::Error for ConfigError {}

impl ConfigError {
    pub fn config_source(&self) -> ConfigSource {
        match self {
            Self::Read { config_source, .. } | Self::MalformedToml { config_source, .. } => {
                config_source.clone()
            }
        }
    }
}
