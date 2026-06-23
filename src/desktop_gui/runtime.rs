use std::sync::Arc;

use tauri::Emitter;

use crate::app::runtime::DesktopRuntime;
use crate::app::AppLiveUpdate;

use super::state::DesktopState;

pub(crate) fn spawn_desktop_runtime(app: tauri::AppHandle, state: Arc<DesktopState>) {
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
