use std::path::{Path, PathBuf};
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
use tokio::sync::{watch, RwLock};

struct DesktopState {
    config: RwLock<RuntimeConfig>,
    config_source: RwLock<ConfigSource>,
    runtime: RwLock<Option<Arc<DesktopRuntime>>>,
    startup_error: RwLock<Option<String>>,
    startup_signal: watch::Sender<()>,
}

impl DesktopState {
    fn new(config: RuntimeConfig, config_source: ConfigSource, startup_error: Option<String>) -> Self {
        let (startup_signal, _receiver) = watch::channel(());
        Self {
            config: RwLock::new(config),
            config_source: RwLock::new(config_source),
            runtime: RwLock::new(None),
            startup_error: RwLock::new(startup_error),
            startup_signal,
        }
    }

    fn notify_startup_changed(&self) {
        self.startup_signal.send_replace(());
    }
}

#[tauri::command]
async fn load_snapshot(state: State<'_, Arc<DesktopState>>) -> Result<AppSnapshot, String> {
    let mut startup_signal = state.startup_signal.subscribe();
    loop {
        if let Some(runtime) = state.runtime.read().await.clone() {
            return Ok(runtime.snapshot().await);
        }
        if let Some(message) = state.startup_error.read().await.clone() {
            return Err(message);
        }
        if startup_signal.changed().await.is_err() {
            return Err("Desktop monitor startup channel closed".to_string());
        }
    }
}

#[tauri::command]
async fn load_config(state: State<'_, Arc<DesktopState>>) -> Result<ConfigApiView, String> {
    let config = state.config.read().await;
    Ok(config_api_view(&config))
}

#[tauri::command]
async fn save_config(
    state: State<'_, Arc<DesktopState>>,
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
            let app_config_dir = app.path().app_config_dir().map_err(|_error| {
                Box::<dyn std::error::Error>::from("Config directory could not be resolved")
            })?;
            let config_path = desktop_config_path(&app_config_dir);
            let startup = load_desktop_startup(&config_path);
            let should_start_runtime = startup.startup_error.is_none();
            let state = Arc::new(DesktopState::new(
                startup.config,
                startup.config_source,
                startup.startup_error,
            ));
            app.manage(Arc::clone(&state));
            if should_start_runtime {
                spawn_desktop_runtime(app.handle().clone(), state);
            }
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
    startup_error: Option<String>,
}

fn desktop_config_path(app_config_dir: &Path) -> PathBuf {
    match std::env::current_exe() {
        Ok(exe_path) => desktop_config_path_from_exe(app_config_dir, &exe_path),
        Err(_error) => app_config_dir.join("config.toml"),
    }
}

fn desktop_config_path_from_exe(app_config_dir: &Path, exe_path: &Path) -> PathBuf {
    exe_path
        .parent()
        .map_or_else(|| app_config_dir.join("config.toml"), |dir| {
            dir.join("config.toml")
        })
}

fn load_desktop_startup(config_path: &Path) -> DesktopStartup {
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

fn spawn_desktop_runtime(app: tauri::AppHandle, state: Arc<DesktopState>) {
    tauri::async_runtime::spawn(async move {
        let config = state.config.read().await.clone();
        match DesktopRuntime::start(config).await {
            Ok(runtime) => {
                let runtime = Arc::new(runtime);
                {
                    let mut startup_error = state.startup_error.write().await;
                    *startup_error = None;
                }
                {
                    let mut stored_runtime = state.runtime.write().await;
                    *stored_runtime = Some(runtime.clone());
                }
                spawn_event_bridge(app, runtime);
            }
            Err(_error) => {
                let mut startup_error = state.startup_error.write().await;
                *startup_error = Some("Desktop monitor startup failed".to_string());
            }
        }
        state.notify_startup_changed();
    });
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

    use super::{
        desktop_config_path_from_exe, frontend_safe_config_load_error,
        frontend_safe_config_write_error,
    };

    #[test]
    fn desktop_config_path_prefers_exe_sibling_config() {
        let app_config_dir = PathBuf::from("/home/user/AppData/Roaming/dev.ed-sentry.gui");
        let exe_path = PathBuf::from("C:/Users/user/Downloads/ed-sentry/ed-sentry-gui.exe");

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
