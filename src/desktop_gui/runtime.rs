use std::sync::atomic::Ordering;
use std::sync::Arc;

use async_trait::async_trait;
use tauri::Emitter;

use crate::app::runtime::DesktopRuntime;
use crate::app::AppLiveUpdate;
use crate::config::RuntimeConfig;
use crate::web::RuntimeConfigApplier;

use super::config::{frontend_safe_runtime_startup_error, load_desktop_startup};
use super::state::DesktopState;

struct DesktopStateConfigObserver {
    state: Arc<DesktopState>,
}

#[async_trait]
impl RuntimeConfigApplier for DesktopStateConfigObserver {
    async fn apply_runtime_config(&self, config: &RuntimeConfig) {
        *self.state.config.write().await = config.clone();
        *self.state.config_source.write().await = config.config_source.clone();
        self.state
            .config_load_failed
            .store(false, Ordering::Release);
    }
}

pub(crate) fn spawn_desktop_runtime(app: tauri::AppHandle, state: Arc<DesktopState>) {
    tauri::async_runtime::spawn(async move {
        run_desktop_runtime_attempt(state, move |runtime| {
            spawn_event_bridge(app, runtime);
        })
        .await;
    });
}

pub(crate) async fn prepare_retry_state(state: &DesktopState) -> Result<bool, String> {
    if state.runtime.read().await.is_some() {
        return Ok(false);
    }
    if state.starting.swap(true, Ordering::AcqRel) {
        return Ok(false);
    }

    if state.config_load_failed.load(Ordering::Acquire) {
        if let Some(config_path) = &state.config_path {
            let startup = load_desktop_startup(config_path);
            if let Some(error) = startup.startup_error {
                *state.startup_error.write().await = Some(error.clone());
                state.starting.store(false, Ordering::Release);
                state.notify_startup_changed();
                return Err(error);
            }
            *state.config.write().await = startup.config;
            *state.config_source.write().await = startup.config_source;
            state.config_load_failed.store(false, Ordering::Release);
        }
    }

    *state.startup_error.write().await = None;
    state.notify_startup_changed();
    Ok(true)
}

pub(crate) async fn wait_for_desktop_runtime(
    state: &DesktopState,
) -> Result<Arc<DesktopRuntime>, String> {
    let mut startup_signal = state.startup_signal.subscribe();
    loop {
        if let Some(runtime) = state.runtime.read().await.clone() {
            return Ok(runtime);
        }
        if !state.is_starting() {
            if let Some(message) = state.startup_error.read().await.clone() {
                return Err(message);
            }
            return Err("Desktop monitor runtime is not started".to_string());
        }
        if startup_signal.changed().await.is_err() {
            return Err("Desktop monitor startup channel closed".to_string());
        }
    }
}

pub(crate) async fn run_desktop_runtime_attempt<F>(state: Arc<DesktopState>, on_started: F)
where
    F: FnOnce(Arc<DesktopRuntime>) + Send + 'static,
{
    let config = state.config.read().await.clone();
    let observer: Arc<dyn RuntimeConfigApplier> = Arc::new(DesktopStateConfigObserver {
        state: Arc::clone(&state),
    });
    match DesktopRuntime::start_with_observer(config.clone(), Some(observer)).await {
        Ok(runtime) => {
            let runtime = Arc::new(runtime);
            {
                let mut startup_error = state.startup_error.write().await;
                *startup_error = None;
            }
            {
                let mut running_config = state.running_config.write().await;
                *running_config = Some(config);
            }
            {
                let mut stored_runtime = state.runtime.write().await;
                *stored_runtime = Some(runtime.clone());
            }
            state.starting.store(false, Ordering::Release);
            on_started(runtime);
        }
        Err(error) => {
            let message = frontend_safe_runtime_startup_error(&error, &config);
            let mut startup_error = state.startup_error.write().await;
            *startup_error = Some(message);
            state.starting.store(false, Ordering::Release);
        }
    }
    state.notify_startup_changed();
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use crate::config::{AppConfig, CliConfigOverrides, ConfigSource};

    use super::{prepare_retry_state, run_desktop_runtime_attempt, wait_for_desktop_runtime};
    use crate::desktop_gui::state::DesktopState;

    #[tokio::test]
    async fn desktop_retry_reruns_startup_and_recovers_after_journal_folder_fix() {
        let empty_dir = tempfile::tempdir().unwrap();
        let valid_dir = tempfile::tempdir().unwrap();
        let journal_path = valid_dir.path().join("Journal.2035-01-08T140000.01.log");
        fs::write(
            &journal_path,
            r#"{"timestamp":"2035-01-08T14:00:00Z","event":"Commander","Name":"Cmdr Retry"}"#,
        )
        .unwrap();

        let mut initial_config = AppConfig::default()
            .into_runtime_with_source(ConfigSource::InMemory, &CliConfigOverrides::default());
        initial_config.journal.folder = empty_dir.path().to_string_lossy().into_owned();
        let state = Arc::new(DesktopState::new(
            None,
            initial_config,
            ConfigSource::InMemory,
            None,
            true,
        ));

        run_desktop_runtime_attempt(Arc::clone(&state), |_| {}).await;
        let first_error = wait_for_desktop_runtime(&state).await.err().unwrap();
        assert_eq!(
            first_error,
            "Desktop monitor startup failed: no Journal.*.log files were found in the configured Journal folder"
        );
        assert!(!first_error.contains(empty_dir.path().to_string_lossy().as_ref()));

        state.config.write().await.journal.folder = valid_dir.path().to_string_lossy().into_owned();
        let should_spawn = prepare_retry_state(&state).await.unwrap();
        assert!(should_spawn);
        tokio::spawn(run_desktop_runtime_attempt(Arc::clone(&state), |_| {}));

        let runtime = wait_for_desktop_runtime(&state).await.unwrap();
        let snapshot = runtime.snapshot().await;
        assert_eq!(snapshot.session.commander.as_deref(), Some("Cmdr Retry"));
    }

    #[tokio::test]
    async fn desktop_save_config_applies_live_monitor_settings_and_marks_restart_sections() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let journal_path = temp_dir.path().join("Journal.2035-01-08T150000.01.log");
        fs::write(
            &journal_path,
            r#"{"timestamp":"2035-01-08T15:00:00Z","event":"Commander","Name":"Cmdr Save"}"#,
        )
        .unwrap();
        fs::write(
            &config_path,
            format!(
                "[journal]\nfolder = {:?}\n\n[web]\nenabled = false\nhost = \"127.0.0.1\"\nport = 8765\n",
                temp_dir.path().display().to_string()
            ),
        )
        .unwrap();

        let loaded = crate::config::AppConfig::load_tauri_from_path(&config_path).unwrap();
        let source = loaded.source.clone();
        let initial_config = loaded
            .config
            .into_runtime_with_source(source.clone(), &CliConfigOverrides::default());
        let state = Arc::new(DesktopState::new(
            Some(config_path),
            initial_config,
            source,
            None,
            true,
        ));
        run_desktop_runtime_attempt(Arc::clone(&state), |_| {}).await;
        let runtime = wait_for_desktop_runtime(&state).await.unwrap();

        let live_view = crate::desktop_gui::save_desktop_config(
            &state,
            crate::app::EditableConfigUpdate {
                monitor: Some(crate::app::MonitorConfigEdit {
                    use_utc: None,
                    live_status: None,
                    dynamic_title: None,
                    warn_kill_rate: Some(42),
                    warn_kill_rate_delay_minutes: None,
                    warn_no_kills_minutes: Some(13),
                    warn_no_kills_initial_minutes: None,
                    warn_cooldown_minutes: None,
                    duplicate_max: None,
                    pirate_names: None,
                    bounty_faction: None,
                    bounty_value: None,
                    extended_stats: None,
                    min_scan_level: None,
                    poll_interval_ms: Some(250),
                }),
                ..crate::app::EditableConfigUpdate::default()
            },
        )
        .await
        .unwrap();
        assert!(!live_view.restart_required);
        assert!(live_view.restart_required_sections.is_empty());
        let snapshot = runtime.snapshot().await;
        assert_eq!(snapshot.thresholds.warn_kill_rate_per_hour, 42);
        assert_eq!(snapshot.thresholds.warn_no_kills_minutes, 13);

        let restart_view = crate::desktop_gui::save_desktop_config(
            &state,
            crate::app::EditableConfigUpdate {
                web: Some(crate::app::WebConfigEdit {
                    enabled: Some(true),
                    host: Some("127.0.0.1".to_string()),
                    port: Some(9123),
                    open_browser: Some(false),
                }),
                ..crate::app::EditableConfigUpdate::default()
            },
        )
        .await
        .unwrap();
        assert!(restart_view.restart_required);
        assert_eq!(restart_view.restart_required_sections, vec!["Web UI"]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn web_save_config_refreshes_desktop_state_and_survives_next_desktop_save() {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        let _env = crate::web::test_env_lock().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let dist_dir = tempfile::tempdir().unwrap();
        fs::write(dist_dir.path().join("index.html"), "<!doctype html>").unwrap();
        std::env::set_var("ED_SENTRY_WEBUI_DIST", dist_dir.path());

        let config_path = temp_dir.path().join("config.toml");
        let journal_path = temp_dir.path().join("Journal.2035-01-08T160000.01.log");
        fs::write(
            &journal_path,
            r#"{"timestamp":"2035-01-08T16:00:00Z","event":"Commander","Name":"Cmdr Bridge"}"#,
        )
        .unwrap();
        fs::write(
            &config_path,
            format!(
                "[journal]\nfolder = {:?}\n\n[monitor]\nwarn_kill_rate = 20\npirate_names = false\n\n[web]\nenabled = true\nhost = \"127.0.0.1\"\nport = 0\nopen_browser = false\n",
                temp_dir.path().display().to_string()
            ),
        )
        .unwrap();

        let loaded = AppConfig::load_tauri_from_path(&config_path).unwrap();
        let source = loaded.source.clone();
        let initial_config = loaded
            .config
            .into_runtime_with_source(source.clone(), &CliConfigOverrides::default());
        let state = Arc::new(DesktopState::new(
            Some(config_path.clone()),
            initial_config,
            source,
            None,
            true,
        ));
        run_desktop_runtime_attempt(Arc::clone(&state), |_| {}).await;
        let runtime = wait_for_desktop_runtime(&state).await.unwrap();
        let port = runtime
            .web_bound_port()
            .expect("embedded WebUI should bind");

        let body = r#"{"monitor":{"warn_kill_rate":55,"poll_interval_ms":300}}"#;
        let request = format!(
            "PUT /api/config HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let response = tokio::task::spawn_blocking(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            stream.write_all(request.as_bytes()).unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            response
        })
        .await
        .unwrap();
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");

        assert_eq!(
            runtime.snapshot().await.thresholds.warn_kill_rate_per_hour,
            55
        );
        let stored_after_web =
            crate::desktop_gui::config::config_api_view(&*state.config.read().await);
        assert_eq!(stored_after_web.config.monitor.warn_kill_rate, 55);
        assert_eq!(stored_after_web.config.monitor.poll_interval_ms, 300);

        let desktop_view = crate::desktop_gui::save_desktop_config(
            &state,
            crate::app::EditableConfigUpdate {
                monitor: Some(crate::app::MonitorConfigEdit {
                    use_utc: None,
                    live_status: None,
                    dynamic_title: None,
                    warn_kill_rate: Some(stored_after_web.config.monitor.warn_kill_rate),
                    warn_kill_rate_delay_minutes: None,
                    warn_no_kills_minutes: None,
                    warn_no_kills_initial_minutes: None,
                    warn_cooldown_minutes: None,
                    duplicate_max: None,
                    pirate_names: Some(true),
                    bounty_faction: None,
                    bounty_value: None,
                    extended_stats: None,
                    min_scan_level: None,
                    poll_interval_ms: Some(stored_after_web.config.monitor.poll_interval_ms),
                }),
                ..crate::app::EditableConfigUpdate::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(desktop_view.config.monitor.warn_kill_rate, 55);
        assert!(desktop_view.config.monitor.pirate_names);

        let reloaded = AppConfig::load_from_path(&config_path).unwrap();
        assert_eq!(reloaded.config.monitor.warn_kill_rate, 55);
        assert_eq!(reloaded.config.monitor.poll_interval_ms, 300);
        assert!(reloaded.config.monitor.pirate_names);

        drop(runtime);
        std::env::remove_var("ED_SENTRY_WEBUI_DIST");
    }
}
