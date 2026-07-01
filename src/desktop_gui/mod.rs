mod config;
mod runtime;
mod state;

use std::sync::Arc;

use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::app::{AppSnapshot, ConfigApiView, EditableConfigUpdate, TunnelStatusView};

use self::config::{config_api_view, load_desktop_startup};
use self::runtime::spawn_desktop_runtime;
use self::state::DesktopState;

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
    let source = state.config_source.read().await.clone();
    let outcome = crate::config::AppConfig::write_update_to_source(&source, &update)
        .map_err(config::frontend_safe_config_write_error)?;
    let runtime_config = outcome.config.into_runtime_with_source(
        outcome.source.clone(),
        &crate::config::CliConfigOverrides::default(),
    );
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
    let mut startup_signal = state.startup_signal.subscribe();
    loop {
        if let Some(runtime) = state.runtime.read().await.clone() {
            return Ok(runtime);
        }
        if let Some(message) = state.startup_error.read().await.clone() {
            return Err(message);
        }
        if startup_signal.changed().await.is_err() {
            return Err("Desktop monitor startup channel closed".to_string());
        }
    }
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
            load_tunnel_status,
            start_tunnel,
            save_config,
            open_external_url
        ])
        .run(tauri::generate_context!("ui/src-tauri/tauri.conf.json"))
}
