use std::path::Path;
use std::sync::Arc;

use ed_sentry::app::runtime::DesktopRuntime;
use ed_sentry::app::{
    AppLiveUpdate, AppSnapshot, ConfigApiView, ConfigEndpointPolicy, EditableConfigUpdate,
    EditableConfigView,
};
use ed_sentry::config::{
    AppConfig, CliConfigOverrides, ConfigError, ConfigSource, ConfigWriteError, RuntimeConfig,
};
use tauri::{Emitter, Manager, State};
use tokio::sync::RwLock;

struct DesktopState {
    config: RwLock<RuntimeConfig>,
    config_source: RwLock<ConfigSource>,
    runtime: RwLock<Option<Arc<DesktopRuntime>>>,
    startup_error: RwLock<Option<String>>,
}

impl DesktopState {
    fn new(
        config: RuntimeConfig,
        config_source: ConfigSource,
        runtime: Option<Arc<DesktopRuntime>>,
        startup_error: Option<String>,
    ) -> Self {
        Self {
            config: RwLock::new(config),
            config_source: RwLock::new(config_source),
            runtime: RwLock::new(runtime),
            startup_error: RwLock::new(startup_error),
        }
    }
}

#[tauri::command]
async fn load_snapshot(state: State<'_, DesktopState>) -> Result<AppSnapshot, String> {
    let runtime = state.runtime.read().await.clone();
    match runtime {
        Some(runtime) => Ok(runtime.snapshot().await),
        None => {
            let message = state
                .startup_error
                .read()
                .await
                .clone()
                .unwrap_or_else(|| "Desktop monitor runtime is not started".to_string());
            Err(message)
        }
    }
}

#[tauri::command]
async fn load_config(state: State<'_, DesktopState>) -> Result<ConfigApiView, String> {
    let config = state.config.read().await;
    Ok(config_api_view(&config))
}

#[tauri::command]
async fn save_config(
    state: State<'_, DesktopState>,
    update: EditableConfigUpdate,
) -> Result<ConfigApiView, String> {
    let source = state.config_source.read().await.clone();
    let outcome = AppConfig::write_update_to_source(&source, &update)
        .map_err(frontend_safe_config_write_error)?;
    let runtime_config = outcome
        .config
        .into_runtime_with_source(outcome.source.clone(), &CliConfigOverrides::default());
    {
        let mut config = state.config.write().await;
        *config = runtime_config;
    }
    {
        let mut config_source = state.config_source.write().await;
        *config_source = outcome.source;
    }
    load_config(state).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app.path().app_config_dir().map_err(|_error| {
                Box::<dyn std::error::Error>::from("Config directory could not be resolved")
            })?;
            let startup = load_desktop_startup(&config_dir);
            if let Some(runtime) = startup.runtime.clone() {
                spawn_event_bridge(app.handle().clone(), runtime);
            }
            app.manage(DesktopState::new(
                startup.config,
                startup.config_source,
                startup.runtime,
                startup.startup_error,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_snapshot,
            load_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running ed-sentry-gui")
}

struct DesktopStartup {
    config: RuntimeConfig,
    config_source: ConfigSource,
    runtime: Option<Arc<DesktopRuntime>>,
    startup_error: Option<String>,
}

fn load_desktop_startup(config_dir: &Path) -> DesktopStartup {
    if let Err(error) = std::fs::create_dir_all(config_dir) {
        let config = AppConfig::default()
            .into_runtime_with_source(ConfigSource::InMemory, &CliConfigOverrides::default());
        return DesktopStartup {
            config,
            config_source: ConfigSource::InMemory,
            runtime: None,
            startup_error: Some(format!(
                "Config directory could not be created: {}",
                safe_io_error(&error)
            )),
        };
    }

    let loaded = match AppConfig::load_tauri_from_dir(config_dir) {
        Ok(loaded) => loaded,
        Err(error) => {
            let source = error.config_source();
            let config = AppConfig::default()
                .into_runtime_with_source(source.clone(), &CliConfigOverrides::default());
            return DesktopStartup {
                config,
                config_source: source,
                runtime: None,
                startup_error: Some(frontend_safe_config_load_error(error)),
            };
        }
    };
    let config_source = loaded.source.clone();
    let config = loaded
        .config
        .into_runtime_with_source(config_source.clone(), &CliConfigOverrides::default());
    let runtime = match tauri::async_runtime::block_on(DesktopRuntime::start(config.clone())) {
        Ok(runtime) => Some(Arc::new(runtime)),
        Err(_error) => {
            return DesktopStartup {
                config,
                config_source,
                runtime: None,
                startup_error: Some("Desktop monitor startup failed".to_string()),
            };
        }
    };

    DesktopStartup {
        config,
        config_source,
        runtime,
        startup_error: None,
    }
}

fn spawn_event_bridge(app: tauri::AppHandle, runtime: Arc<DesktopRuntime>) {
    let subscriber = runtime.event_store().subscribe();
    let _ = app.emit(
        "ed-sentry://dashboard",
        AppLiveUpdate::Snapshot {
            snapshot: Box::new(subscriber.bootstrap.snapshot),
        },
    );
    tauri::async_runtime::spawn_blocking(move || {
        for update in subscriber.live {
            if app.emit("ed-sentry://dashboard", update).is_err() {
                break;
            }
        }
    });
}

fn config_api_view(config: &RuntimeConfig) -> ConfigApiView {
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

fn frontend_safe_config_write_error(error: ConfigWriteError) -> String {
    match error {
        ConfigWriteError::NoWritableTarget => {
            "Config save failed: config source has no writable target".to_string()
        }
        ConfigWriteError::Blocked { reason } => {
            format!("Config save failed: config write blocked by source state: {reason:?}")
        }
        ConfigWriteError::UnsafeRemoteBind { .. } => {
            "Config save failed: web.host is not loopback; remote writes are disabled".to_string()
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

    use ed_sentry::config::{ConfigError, ConfigSource, ConfigWriteError};

    use super::{frontend_safe_config_load_error, frontend_safe_config_write_error};

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
