mod config;
mod runtime;
mod state;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::app::{AppSnapshot, ConfigApiView, EditableConfigUpdate, TunnelStatusView};

use self::config::{config_api_view, load_desktop_startup};
use self::runtime::{prepare_retry_state, spawn_desktop_runtime, wait_for_desktop_runtime};
use self::state::DesktopState;

#[tauri::command]
async fn load_snapshot(state: State<'_, Arc<DesktopState>>) -> Result<AppSnapshot, String> {
    let runtime = wait_for_desktop_runtime(&state).await?;
    Ok(runtime.snapshot().await)
}

#[tauri::command]
async fn retry_startup(
    app: tauri::AppHandle,
    state: State<'_, Arc<DesktopState>>,
) -> Result<AppSnapshot, String> {
    if prepare_retry_state(&state).await? {
        spawn_desktop_runtime(app, Arc::clone(&state));
    }
    let runtime = wait_for_desktop_runtime(&state).await?;
    Ok(runtime.snapshot().await)
}

#[tauri::command]
async fn load_config(state: State<'_, Arc<DesktopState>>) -> Result<ConfigApiView, String> {
    Ok(load_desktop_config(&state).await)
}

pub(crate) async fn load_desktop_config(state: &DesktopState) -> ConfigApiView {
    let config = state.config.read().await.clone();
    let restart_required_sections = desktop_restart_required_sections(state, &config).await;
    let mut view = config_api_view(&config);
    view.restart_required = !restart_required_sections.is_empty();
    view.restart_required_sections = restart_required_sections;
    view
}

#[tauri::command]
async fn load_tunnel_status(
    state: State<'_, Arc<DesktopState>>,
) -> Result<TunnelStatusView, String> {
    let runtime = desktop_runtime(state).await?;
    Ok(runtime.tunnel_status().await.into())
}

#[tauri::command]
async fn start_tunnel(state: State<'_, Arc<DesktopState>>) -> Result<TunnelStatusView, String> {
    let runtime = desktop_runtime(state).await?;
    Ok(runtime.start_tunnel().await.into())
}

#[tauri::command]
async fn save_config(
    state: State<'_, Arc<DesktopState>>,
    update: EditableConfigUpdate,
) -> Result<ConfigApiView, String> {
    save_desktop_config(&state, update).await
}

pub(crate) async fn save_desktop_config(
    state: &DesktopState,
    update: EditableConfigUpdate,
) -> Result<ConfigApiView, String> {
    let _write_guard = crate::config::config_write_lock().lock().await;
    let source = state.config_source.read().await.clone();
    let outcome = crate::config::AppConfig::write_update_to_source(&source, &update)
        .map_err(config::frontend_safe_config_write_error)?;
    let runtime_config = outcome.config.into_runtime_with_source(
        outcome.source.clone(),
        &crate::config::CliConfigOverrides::default(),
    );
    {
        let mut config = state.config.write().await;
        *config = runtime_config.clone();
    }
    {
        let mut config_source = state.config_source.write().await;
        *config_source = outcome.source;
    }
    state.config_load_failed.store(false, Ordering::Release);
    if let Some(runtime) = state.runtime.read().await.clone() {
        runtime.apply_runtime_config(&runtime_config).await;
    }
    let restart_required_sections = desktop_restart_required_sections(state, &runtime_config).await;
    let mut view = config_api_view(&runtime_config);
    view.restart_required = !restart_required_sections.is_empty();
    view.restart_required_sections = restart_required_sections;
    Ok(view)
}

async fn desktop_restart_required_sections(
    state: &DesktopState,
    config: &crate::config::RuntimeConfig,
) -> Vec<String> {
    let Some(runtime) = state.runtime.read().await.clone() else {
        return Vec::new();
    };
    let tunnel_running = runtime.tunnel_is_running().await;
    match state.running_config.read().await.as_ref() {
        Some(running) => crate::app::restart_required_sections(running, config, tunnel_running),
        None => Vec::new(),
    }
}

#[tauri::command]
fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    let parsed_url =
        url::Url::parse(&url).map_err(|error| format!("Invalid external URL: {error}"))?;
    match parsed_url.scheme() {
        "http" | "https" => {}
        _ => return Err("External URL must use http or https".to_string()),
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| format!("Failed to open external URL: {error}"))
}

async fn desktop_runtime(
    state: State<'_, Arc<DesktopState>>,
) -> Result<Arc<crate::app::runtime::DesktopRuntime>, String> {
    wait_for_desktop_runtime(&state).await
}

pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_config_dir = app.path().app_config_dir().map_err(|_error| {
                Box::<dyn std::error::Error>::from("Config directory could not be resolved")
            })?;
            let config_path = config::desktop_config_path(&app_config_dir);
            let startup = load_desktop_startup(&config_path);
            let should_start_runtime = startup.startup_error.is_none();
            let state = Arc::new(DesktopState::new(
                Some(config_path),
                startup.config,
                startup.config_source,
                startup.startup_error,
                should_start_runtime,
            ));
            app.manage(Arc::clone(&state));
            if should_start_runtime {
                spawn_desktop_runtime(app.handle().clone(), state);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_snapshot,
            retry_startup,
            load_config,
            load_tunnel_status,
            start_tunnel,
            save_config,
            open_external_url
        ])
        .run(tauri::generate_context!("ui/src-tauri/tauri.conf.json"))
}
