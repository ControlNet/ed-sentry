use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ConfigSource {
    #[default]
    InMemory,
    Explicit(ConfigPath),
    Implicit(ConfigPath),
    Defaults {
        first_save_target: ConfigPath,
    },
    Tauri {
        target: ConfigPath,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigPath {
    pub path: PathBuf,
    pub write_state: ConfigWriteState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigWriteState {
    Editable,
    FirstSave,
    Blocked(ConfigBlockReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigBlockReason {
    MalformedStrictConfig,
    ReadFailure,
}

impl ConfigPath {
    pub fn editable(path: PathBuf) -> Self {
        Self {
            path,
            write_state: ConfigWriteState::Editable,
        }
    }

    pub fn first_save(path: PathBuf) -> Self {
        Self {
            path,
            write_state: ConfigWriteState::FirstSave,
        }
    }

    pub fn blocked(path: PathBuf, reason: ConfigBlockReason) -> Self {
        Self {
            path,
            write_state: ConfigWriteState::Blocked(reason),
        }
    }
}

pub(super) fn config_source_path(source: &ConfigSource) -> Option<PathBuf> {
    match source {
        ConfigSource::InMemory => None,
        ConfigSource::Explicit(path) | ConfigSource::Implicit(path) => Some(path.path.clone()),
        ConfigSource::Defaults { first_save_target } => Some(first_save_target.path.clone()),
        ConfigSource::Tauri { target } => Some(target.path.clone()),
    }
}

pub(super) fn blocked_source(source: &ConfigSource, reason: ConfigBlockReason) -> ConfigSource {
    match source {
        ConfigSource::InMemory => ConfigSource::InMemory,
        ConfigSource::Explicit(path) => {
            ConfigSource::Explicit(ConfigPath::blocked(path.path.clone(), reason))
        }
        ConfigSource::Implicit(path) => {
            ConfigSource::Implicit(ConfigPath::blocked(path.path.clone(), reason))
        }
        ConfigSource::Defaults { first_save_target } => ConfigSource::Defaults {
            first_save_target: ConfigPath::blocked(first_save_target.path.clone(), reason),
        },
        ConfigSource::Tauri { target } => ConfigSource::Tauri {
            target: ConfigPath::blocked(target.path.clone(), reason),
        },
    }
}

#[cfg(test)]
pub(super) mod test_support {
    use super::*;
    use crate::config::AppConfig;

    pub(in crate::config) fn assert_source_tracks_write_target() {
        let temp_dir = tempfile::tempdir().unwrap();
        let explicit_path = temp_dir.path().join("explicit.toml");
        std::fs::write(&explicit_path, "").unwrap();
        let explicit = AppConfig::load_optional_from_dir(Some(&explicit_path), temp_dir.path())
            .unwrap()
            .source;
        assert_eq!(
            explicit,
            ConfigSource::Explicit(ConfigPath::editable(explicit_path.clone()))
        );

        let implicit_path = temp_dir.path().join("config.toml");
        std::fs::write(&implicit_path, "").unwrap();
        let implicit = AppConfig::load_optional_from_dir(None, temp_dir.path())
            .unwrap()
            .source;
        assert_eq!(
            implicit,
            ConfigSource::Implicit(ConfigPath::editable(implicit_path.clone()))
        );

        std::fs::remove_file(&implicit_path).unwrap();
        let defaults = AppConfig::load_optional_from_dir(None, temp_dir.path())
            .unwrap()
            .source;
        assert_eq!(
            defaults,
            ConfigSource::Defaults {
                first_save_target: ConfigPath::first_save(implicit_path.clone()),
            }
        );

        let malformed_path = temp_dir.path().join("malformed.toml");
        std::fs::write(&malformed_path, "[web\n").unwrap();
        let malformed =
            AppConfig::load_optional_from_dir(Some(&malformed_path), temp_dir.path()).unwrap_err();
        assert_eq!(
            malformed.config_source(),
            ConfigSource::Explicit(ConfigPath::blocked(
                malformed_path,
                ConfigBlockReason::MalformedStrictConfig
            ))
        );

        let missing_path = temp_dir.path().join("missing.toml");
        let read_failure =
            AppConfig::load_optional_from_dir(Some(&missing_path), temp_dir.path()).unwrap_err();
        assert_eq!(
            read_failure.config_source(),
            ConfigSource::Explicit(ConfigPath::blocked(
                missing_path,
                ConfigBlockReason::ReadFailure
            ))
        );

        let tauri_dir = temp_dir.path().join("tauri");
        let tauri = AppConfig::load_tauri_from_dir(&tauri_dir).unwrap().source;
        assert_eq!(
            tauri,
            ConfigSource::Tauri {
                target: ConfigPath::first_save(tauri_dir.join("config.toml")),
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::test_support;

    #[test]
    fn config_source_tracks_write_target() {
        test_support::assert_source_tracks_write_target();
    }
}
