use std::fs;
use std::path::Path;

use toml::Value;

use super::journal::read_journal_config;
use super::log_levels::read_log_levels;
use super::matrix::read_matrix_config;
use super::monitor::read_monitor_config;
use super::source::{blocked_source, config_source_path};
use super::tunnel::read_tunnel_config;
use super::{AppConfig, ConfigBlockReason, ConfigError, ConfigPath, ConfigSource, LoadedConfig};

impl AppConfig {
    pub fn from_toml_str(contents: &str) -> Result<LoadedConfig, ConfigError> {
        Self::from_toml_str_with_source(contents, ConfigSource::InMemory)
    }

    fn from_toml_str_with_source(
        contents: &str,
        source: ConfigSource,
    ) -> Result<LoadedConfig, ConfigError> {
        let value =
            contents
                .parse::<Value>()
                .map_err(|parse_error| ConfigError::MalformedToml {
                    path: config_source_path(&source),
                    config_source: blocked_source(
                        &source,
                        ConfigBlockReason::MalformedStrictConfig,
                    ),
                    source: Box::new(parse_error),
                })?;
        Ok(Self::from_toml_value(&value, source))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<LoadedConfig, ConfigError> {
        let path = path.as_ref();
        Self::load_config_source(
            path,
            ConfigSource::Explicit(ConfigPath::editable(path.to_path_buf())),
        )
    }

    pub fn load_optional(path: Option<&Path>) -> Result<LoadedConfig, ConfigError> {
        Self::load_optional_from_dir(path, Path::new("."))
    }

    pub fn load_optional_from_dir(
        path: Option<&Path>,
        config_dir: &Path,
    ) -> Result<LoadedConfig, ConfigError> {
        match path {
            Some(path) => Self::load_config_source(
                path,
                ConfigSource::Explicit(ConfigPath::editable(path.to_path_buf())),
            ),
            None => Self::load_implicit_or_defaults(config_dir),
        }
    }

    pub fn load_tauri_from_dir(config_dir: &Path) -> Result<LoadedConfig, ConfigError> {
        Self::load_tauri_from_path(config_dir.join("config.toml"))
    }

    pub fn load_tauri_from_path(path: impl AsRef<Path>) -> Result<LoadedConfig, ConfigError> {
        let path = path.as_ref();
        let source = ConfigSource::Tauri {
            target: ConfigPath::editable(path.to_path_buf()),
        };
        if path.exists() {
            Self::load_config_source(path, source)
        } else {
            Ok(LoadedConfig {
                config: Self::default(),
                warnings: Vec::new(),
                source: ConfigSource::Tauri {
                    target: ConfigPath::first_save(path.to_path_buf()),
                },
            })
        }
    }

    fn load_config_source(path: &Path, source: ConfigSource) -> Result<LoadedConfig, ConfigError> {
        let contents = fs::read_to_string(path).map_err(|read_error| ConfigError::Read {
            path: path.to_path_buf(),
            config_source: blocked_source(&source, ConfigBlockReason::ReadFailure),
            source: read_error,
        })?;
        Self::from_toml_str_with_source(&contents, source)
    }

    fn load_implicit_or_defaults(config_dir: &Path) -> Result<LoadedConfig, ConfigError> {
        let implicit_path = config_dir.join("config.toml");
        if implicit_path.exists() {
            Self::load_config_source(
                &implicit_path,
                ConfigSource::Implicit(ConfigPath::editable(implicit_path.clone())),
            )
        } else {
            Ok(LoadedConfig {
                config: Self::default(),
                warnings: Vec::new(),
                source: ConfigSource::Defaults {
                    first_save_target: ConfigPath::first_save(implicit_path),
                },
            })
        }
    }

    fn from_toml_value(value: &Value, source: ConfigSource) -> LoadedConfig {
        let mut config = Self::default();
        let mut warnings = Vec::new();

        if let Some(journal) = value.get("journal") {
            if let Some(table) = journal.as_table() {
                read_journal_config(table, &mut config.journal, &mut warnings);
            } else {
                warnings.push(
                    "config key journal has wrong type; using defaults for section".to_string(),
                );
            }
        }

        if let Some(monitor) = value.get("monitor") {
            if let Some(table) = monitor.as_table() {
                read_monitor_config(table, &mut config.monitor, &mut warnings);
            } else {
                warnings.push(
                    "config key monitor has wrong type; using defaults for section".to_string(),
                );
            }
        }

        if let Some(log_levels) = value.get("log_levels") {
            if let Some(table) = log_levels.as_table() {
                read_log_levels(table, &mut config.log_levels, &mut warnings);
            } else {
                warnings.push(
                    "config key log_levels has wrong type; using defaults for section".to_string(),
                );
            }
        }

        if let Some(matrix) = value.get("matrix") {
            if let Some(table) = matrix.as_table() {
                config.matrix = read_matrix_config(table, &mut warnings);
            } else {
                warnings.push(
                    "config key matrix has wrong type; using defaults for section".to_string(),
                );
            }
        }

        if let Some(tunnel) = value.get("tunnel") {
            if let Some(table) = tunnel.as_table() {
                read_tunnel_config(table, &mut config.tunnel, &mut warnings);
            } else {
                warnings.push(
                    "config key tunnel has wrong type; using defaults for section".to_string(),
                );
            }
        }

        if let Some(web) = value.get("web") {
            if let Some(table) = web.as_table() {
                super::web::read_web_config(table, &mut config.web, &mut warnings);
            } else {
                warnings
                    .push("config key web has wrong type; using defaults for section".to_string());
            }
        }

        LoadedConfig {
            config,
            warnings,
            source,
        }
    }
}
