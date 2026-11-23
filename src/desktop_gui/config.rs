use std::path::{Path, PathBuf};

use crate::app::runtime::RuntimeError;
use crate::app::{ConfigApiView, ConfigEndpointPolicy, EditableConfigView};
use crate::config::{
    AppConfig, CliConfigOverrides, ConfigError, ConfigSource, ConfigWriteError, RuntimeConfig,
};
use crate::journal::JournalError;
use crate::text::line_safe;

pub(crate) struct DesktopStartup {
    pub(crate) config: RuntimeConfig,
    pub(crate) config_source: ConfigSource,
    pub(crate) startup_error: Option<String>,
}

pub(crate) fn desktop_config_path(app_config_dir: &Path) -> PathBuf {
    match std::env::current_exe() {
        Ok(exe_path) => desktop_config_path_from_exe(app_config_dir, &exe_path),
        Err(_error) => app_config_dir.join("config.toml"),
    }
}

fn desktop_config_path_from_exe(app_config_dir: &Path, exe_path: &Path) -> PathBuf {
    exe_path.parent().map_or_else(
        || app_config_dir.join("config.toml"),
        |dir| dir.join("config.toml"),
    )
}

pub(crate) fn load_desktop_startup(config_path: &Path) -> DesktopStartup {
    if let Some(config_dir) = config_path.parent() {
        if let Err(error) = std::fs::create_dir_all(config_dir) {
            let config = AppConfig::default()
                .into_runtime_with_source(ConfigSource::InMemory, &CliConfigOverrides::default());
            return DesktopStartup {
                config,
                config_source: ConfigSource::InMemory,
                startup_error: Some(format!(
                    "Config directory could not be created: {}",
                    safe_io_error(&error)
                )),
            };
        }
    }

    let loaded = match AppConfig::load_tauri_from_path(config_path) {
        Ok(loaded) => loaded,
        Err(error) => {
            let source = error.config_source();
            let config = AppConfig::default()
                .into_runtime_with_source(source.clone(), &CliConfigOverrides::default());
            return DesktopStartup {
                config,
                config_source: source,
                startup_error: Some(frontend_safe_config_load_error(error)),
            };
        }
    };
    let config_source = loaded.source.clone();
    let config = loaded
        .config
        .into_runtime_with_source(config_source.clone(), &CliConfigOverrides::default());

    DesktopStartup {
        config,
        config_source,
        startup_error: None,
    }
}

pub(crate) fn config_api_view(config: &RuntimeConfig) -> ConfigApiView {
    ConfigApiView {
        version: 1,
        config: EditableConfigView::from_runtime_config(config),
        policy: ConfigEndpointPolicy {
            state_changing_enabled: true,
            state_changing_reason: "enabled for desktop config file".to_string(),
            remote_bind: false,
        },
        restart_required: false,
        restart_required_sections: Vec::new(),
    }
}

fn frontend_safe_config_load_error(error: ConfigError) -> String {
    match error {
        ConfigError::Read { source, .. } => {
            format!(
                "Config load failed: config file could not be read: {}",
                safe_io_error(&source)
            )
        }
        ConfigError::MalformedToml { .. } => {
            "Config load failed: malformed TOML config".to_string()
        }
    }
}

pub(crate) fn frontend_safe_config_write_error(error: ConfigWriteError) -> String {
    match error {
        ConfigWriteError::NoWritableTarget => {
            "Config save failed: config source has no writable target".to_string()
        }
        ConfigWriteError::Blocked { reason } => {
            format!("Config save failed: config write blocked by source state: {reason:?}")
        }
        ConfigWriteError::UnsafeRemoteBind { .. } => {
            "Config save failed: invalid web.host".to_string()
        }
        ConfigWriteError::InvalidUpdate { reason } => {
            format!("Config save failed: {reason}")
        }
        ConfigWriteError::MalformedToml { .. } => {
            "Config save failed: malformed TOML config".to_string()
        }
        ConfigWriteError::Io { source, .. } => {
            format!(
                "Config save failed: config file could not be written: {}",
                safe_io_error(&source)
            )
        }
    }
}

fn safe_io_error(error: &std::io::Error) -> &'static str {
    match error.kind() {
        std::io::ErrorKind::PermissionDenied => "permission denied",
        _ => "I/O error",
    }
}

fn safe_journal_io_error(error: &std::io::Error) -> &'static str {
    match error.kind() {
        std::io::ErrorKind::NotFound => "not found",
        std::io::ErrorKind::PermissionDenied => "permission denied",
        _ => "I/O error",
    }
}

pub(crate) fn frontend_safe_runtime_startup_error(
    error: &RuntimeError,
    config: &RuntimeConfig,
) -> String {
    if let Some(journal_error) = error.journal_error() {
        return match journal_error {
            JournalError::DefaultPathUnavailable => {
                "Desktop monitor startup failed: Journal folder is not configured and the default Saved Games folder is unavailable".to_string()
            }
            JournalError::NoJournalFiles { .. } => {
                "Desktop monitor startup failed: no Journal.*.log files were found in the configured Journal folder".to_string()
            }
            JournalError::DirectoryRead { source, .. } => format!(
                "Desktop monitor startup failed: configured Journal folder could not be read: {}",
                safe_journal_io_error(source)
            ),
            JournalError::MetadataRead { source, .. } => format!(
                "Desktop monitor startup failed: Journal metadata could not be read: {}",
                safe_journal_io_error(source)
            ),
            JournalError::FileOpen { source, .. } => format!(
                "Desktop monitor startup failed: Journal file could not be opened: {}",
                safe_journal_io_error(source)
            ),
            JournalError::FileRead { source, .. } => format!(
                "Desktop monitor startup failed: Journal file could not be read: {}",
                safe_journal_io_error(source)
            ),
            JournalError::Callback { .. } => {
                "Desktop monitor startup failed: Journal stream callback failed".to_string()
            }
        };
    }

    let sanitized = sanitize_runtime_error_detail(&error.to_string(), config);
    if sanitized.is_empty() {
        "Desktop monitor startup failed".to_string()
    } else {
        format!("Desktop monitor startup failed: {sanitized}")
    }
}

fn sanitize_runtime_error_detail(message: &str, config: &RuntimeConfig) -> String {
    let mut sanitized = line_safe(message);
    if let Some(token) = config
        .matrix
        .as_ref()
        .and_then(|matrix| matrix.access_token.as_deref())
        .filter(|token| !token.is_empty())
    {
        sanitized = sanitized.replace(token, "[redacted]");
    }
    if !config.tunnel.config_password.is_empty() {
        sanitized = sanitized.replace(&config.tunnel.config_password, "[redacted]");
    }
    if !config.journal.folder.is_empty() {
        sanitized = sanitized.replace(&config.journal.folder, "[redacted path]");
    }
    if let Some(set_file) = &config.set_file {
        let set_file_display = set_file.to_string_lossy();
        if !set_file_display.is_empty() {
            sanitized = sanitized.replace(set_file_display.as_ref(), "[redacted path]");
        }
    }
    sanitized
        .split_whitespace()
        .map(|token| {
            if looks_like_absolute_path(token) {
                "[redacted path]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn looks_like_absolute_path(token: &str) -> bool {
    let trimmed =
        token.trim_matches(|ch: char| matches!(ch, '"' | '\'' | '(' | ')' | ',' | ':' | ';'));
    if trimmed.starts_with('/') && trimmed.len() > 1 {
        return true;
    }
    let bytes = trimmed.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::{ConfigError, ConfigSource, ConfigWriteError};

    use super::{
        desktop_config_path_from_exe, frontend_safe_config_load_error,
        frontend_safe_config_write_error,
    };

    #[test]
    fn desktop_config_path_prefers_exe_sibling_config() {
        let app_config_dir = PathBuf::from("/home/user/AppData/Roaming/dev.ed-sentry.gui");
        let exe_path = PathBuf::from("C:/Users/user/Downloads/ed-sentry/ed-sentry.exe");

        assert_eq!(
            desktop_config_path_from_exe(&app_config_dir, &exe_path),
            PathBuf::from("C:/Users/user/Downloads/ed-sentry/config.toml")
        );
    }

    #[test]
    fn config_errors_omit_native_paths() {
        const CONFIG_PATH: &str = "/var/tmp/ed-sentry-test/config.toml";

        let load_message = frontend_safe_config_load_error(ConfigError::Read {
            path: PathBuf::from(CONFIG_PATH),
            config_source: ConfigSource::InMemory,
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "fixture"),
        });

        assert_eq!(
            load_message,
            "Config load failed: config file could not be read: permission denied"
        );
        assert!(!load_message.contains(CONFIG_PATH) && !load_message.contains("config.toml"));

        let save_message = frontend_safe_config_write_error(ConfigWriteError::Io {
            path: PathBuf::from(CONFIG_PATH),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "fixture"),
        });

        assert_eq!(
            save_message,
            "Config save failed: config file could not be written: permission denied"
        );
        assert!(!save_message.contains(CONFIG_PATH) && !save_message.contains("config.toml"));

        let invalid_message = frontend_safe_config_write_error(ConfigWriteError::InvalidUpdate {
            reason: "monitor.warn_cooldown_minutes must be between 1 and 1440",
        });
        assert_eq!(
            invalid_message,
            "Config save failed: monitor.warn_cooldown_minutes must be between 1 and 1440"
        );
    }

    #[test]
    fn runtime_startup_errors_report_reason_without_paths_or_secrets() {
        let private_dir = PathBuf::from("/var/tmp/ed-sentry-private-journals");
        let missing_error = super::frontend_safe_runtime_startup_error(
            &crate::app::runtime::RuntimeError::from(
                crate::journal::JournalError::NoJournalFiles {
                    folder: private_dir.clone(),
                },
            ),
            &crate::config::RuntimeConfig::default(),
        );
        assert_eq!(
            missing_error,
            "Desktop monitor startup failed: no Journal.*.log files were found in the configured Journal folder"
        );
        assert!(!missing_error.contains("/var/tmp"));

        let unreadable_error = super::frontend_safe_runtime_startup_error(
            &crate::app::runtime::RuntimeError::from(crate::journal::JournalError::DirectoryRead {
                path: private_dir,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
            }),
            &crate::config::RuntimeConfig::default(),
        );
        assert_eq!(
            unreadable_error,
            "Desktop monitor startup failed: configured Journal folder could not be read: not found"
        );
        assert!(!unreadable_error.contains("/var/tmp"));

        let mut config = crate::config::RuntimeConfig::default();
        config.matrix = Some(crate::config::MatrixConfig {
            enabled: true,
            homeserver: Some("https://matrix.invalid".to_string()),
            room_id: Some("!room:matrix.invalid".to_string()),
            access_token: Some("fixture-secret-matrix-token".to_string()),
            mention_user_id: None,
            status_update_interval_seconds: 60,
        });
        config.tunnel.config_password = "fixture-tunnel-secret".to_string();
        let generic_error = super::frontend_safe_runtime_startup_error(
            &crate::app::runtime::RuntimeError::new(
                "auth failed with fixture-secret-matrix-token and fixture-tunnel-secret at C:\\Users\\Cmdr\\Saved Games\\Journal.log",
            ),
            &config,
        );
        assert!(!generic_error.contains("fixture-secret-matrix-token"));
        assert!(!generic_error.contains("fixture-tunnel-secret"));
        assert!(!generic_error.contains("C:\\Users\\Cmdr"));
    }
}
