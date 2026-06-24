use std::path::{Path, PathBuf};

use crate::app::{ConfigApiView, ConfigEndpointPolicy, EditableConfigView};
use crate::config::{
    AppConfig, CliConfigOverrides, ConfigError, ConfigSource, ConfigWriteError, RuntimeConfig,
};

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
    }
}
