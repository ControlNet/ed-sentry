use std::fmt;
use std::fs;
use std::path::PathBuf;

use toml_edit::DocumentMut;

use crate::app::EditableConfigUpdate;

use super::{AppConfig, ConfigBlockReason, ConfigPath, ConfigSource, ConfigWriteState};

mod apply;
mod atomic;

use apply::{apply_update, validate_update};
use atomic::write_document_atomically;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigWriteOutcome {
    pub config: AppConfig,
    pub source: ConfigSource,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum ConfigWriteError {
    NoWritableTarget,
    Blocked {
        reason: ConfigBlockReason,
    },
    UnsafeRemoteBind {
        host: String,
    },
    InvalidUpdate {
        reason: &'static str,
    },
    MalformedToml {
        path: PathBuf,
        source: Box<toml_edit::TomlError>,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl fmt::Display for ConfigWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoWritableTarget => formatter.write_str("config source has no writable target"),
            Self::Blocked { reason } => {
                write!(
                    formatter,
                    "config write blocked by source state: {reason:?}"
                )
            }
            Self::UnsafeRemoteBind { host } => write!(
                formatter,
                "web.host {host} is not loopback; remote WebUI config writes are disabled"
            ),
            Self::InvalidUpdate { reason } => write!(formatter, "invalid config update: {reason}"),
            Self::MalformedToml { path, source } => {
                write!(
                    formatter,
                    "malformed TOML config {}: {source}",
                    path.display()
                )
            }
            Self::Io { path, source } => {
                write!(
                    formatter,
                    "failed to write config {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for ConfigWriteError {}

impl AppConfig {
    pub fn write_update_to_source(
        source: &ConfigSource,
        update: &EditableConfigUpdate,
    ) -> Result<ConfigWriteOutcome, ConfigWriteError> {
        let target = write_target(source)?;
        validate_update(update)?;
        let existing = match target.path.exists() {
            true => fs::read_to_string(&target.path).map_err(|error| ConfigWriteError::Io {
                path: target.path.clone(),
                source: error,
            })?,
            false => String::new(),
        };
        let mut document =
            existing
                .parse::<DocumentMut>()
                .map_err(|error| ConfigWriteError::MalformedToml {
                    path: target.path.clone(),
                    source: Box::new(error),
                })?;
        apply_update(&mut document, update);
        if let Some(parent) = target.path.parent() {
            fs::create_dir_all(parent).map_err(|error| ConfigWriteError::Io {
                path: target.path.clone(),
                source: error,
            })?;
        }
        write_document_atomically(&target.path, &document.to_string())?;
        let loaded = Self::load_from_path(&target.path).map_err(|error| ConfigWriteError::Io {
            path: target.path.clone(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()),
        })?;
        Ok(ConfigWriteOutcome {
            config: loaded.config,
            source: replace_source_path(source, target.path.clone()),
            path: target.path.clone(),
        })
    }
}

fn write_target(source: &ConfigSource) -> Result<&ConfigPath, ConfigWriteError> {
    let path = match source {
        ConfigSource::InMemory => return Err(ConfigWriteError::NoWritableTarget),
        ConfigSource::Explicit(path) | ConfigSource::Implicit(path) => path,
        ConfigSource::Defaults { first_save_target } => first_save_target,
        ConfigSource::Tauri { target } => target,
    };
    match &path.write_state {
        ConfigWriteState::Editable | ConfigWriteState::FirstSave => Ok(path),
        ConfigWriteState::Blocked(reason) => Err(ConfigWriteError::Blocked {
            reason: reason.clone(),
        }),
    }
}

fn replace_source_path(source: &ConfigSource, path: PathBuf) -> ConfigSource {
    match source {
        ConfigSource::InMemory => ConfigSource::InMemory,
        ConfigSource::Explicit(_) => ConfigSource::Explicit(ConfigPath::editable(path)),
        ConfigSource::Implicit(_) | ConfigSource::Defaults { .. } => {
            ConfigSource::Implicit(ConfigPath::editable(path))
        }
        ConfigSource::Tauri { .. } => ConfigSource::Tauri {
            target: ConfigPath::editable(path),
        },
    }
}
