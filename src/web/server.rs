use std::net::SocketAddr;

use chrono::Utc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::app::{
    AppEventStore, AppSnapshot, MatrixStartupStatus, WebStartupStatus, WebStatusView,
};
use crate::config::{RuntimeConfig, WebConfig};
use crate::mission::MissionTracker;
use crate::state::SessionState;
use crate::text::line_safe;

use super::assets::resolve_assets;
use super::policy::{default_runtime_for_web_config, router, WebApiState};
use super::tunnel_state::WebTunnelState;

pub struct WebServer {
    status: WebStartupStatus,
    local_addr: Option<SocketAddr>,
    warnings: Vec<String>,
    handle: Option<JoinHandle<()>>,
    tunnel: Option<WebTunnelState>,
}

impl WebServer {
    pub fn disabled() -> Self {
        Self {
            status: WebStartupStatus::disabled(),
            local_addr: None,
            warnings: Vec::new(),
            handle: None,
            tunnel: None,
        }
    }

    pub fn status(&self) -> WebStatusView {
        self.status.clone().into()
    }

    pub fn startup_status(&self) -> WebStartupStatus {
        self.status.clone()
    }

    pub fn bound_port(&self) -> Option<u16> {
        self.local_addr.map(|addr| addr.port())
    }

    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub(crate) fn tunnel(&self) -> Option<WebTunnelState> {
        self.tunnel.clone()
    }
}

impl Drop for WebServer {
    fn drop(&mut self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }
}

pub async fn start(config: &WebConfig) -> WebServer {
    let runtime_config = default_runtime_for_web_config(config);
    let snapshot = AppSnapshot::from_state(
        &SessionState::default(),
        &MissionTracker::new(),
        Utc::now(),
        MatrixStartupStatus::disabled(),
        WebStartupStatus::from_current_runtime_config(&runtime_config),
    );
    start_with_state(&runtime_config, AppEventStore::new(snapshot)).await
}

pub async fn start_with_state(config: &RuntimeConfig, events: AppEventStore) -> WebServer {
    if !config.web.enabled {
        return WebServer::disabled();
    }

    let Some(asset_root) = resolve_assets() else {
        return warning_server(
            "WebUI assets not found; checked ED_SENTRY_WEBUI_DIST, executable sibling webui, and ui/dist",
        );
    };
    let bind_address = bind_address(&config.web.host, config.web.port);
    let listener = match TcpListener::bind(&bind_address).await {
        Ok(listener) => listener,
        Err(error) => {
            return warning_server(format!(
                "WebUI bind failed on {}: {}",
                line_safe(&bind_address),
                line_safe(&error.to_string())
            ));
        }
    };
    let local_addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(error) => {
            return warning_server(format!(
                "WebUI bind failed on {}: {}",
                line_safe(&bind_address),
                line_safe(&error.to_string())
            ));
        }
    };
    let mut warnings = startup_warnings(&config.web);
    let url = web_url(&config.web.host, local_addr.port());
    let status = WebStartupStatus::running(local_addr.to_string(), url, Utc::now());
    let tunnel = match WebTunnelState::for_config(Some(local_addr.port()), config) {
        Ok(tunnel) => tunnel,
        Err(_error) => return warning_server("WebUI tunnel auth could not be initialized"),
    };
    let app = router(WebApiState::new(
        asset_root,
        config.web.host.clone(),
        config.clone(),
        events,
        status.clone(),
        tunnel.clone(),
    ));
    let handle = tokio::spawn(async move {
        if let Err(error) = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            eprintln!(
                "Warning: WebUI server stopped: {}",
                line_safe(&error.to_string())
            );
        }
    });

    WebServer {
        status,
        local_addr: Some(local_addr),
        warnings: std::mem::take(&mut warnings),
        handle: Some(handle),
        tunnel: Some(tunnel),
    }
}

fn startup_warnings(config: &WebConfig) -> Vec<String> {
    let mut warnings = Vec::new();
    if config.open_browser {
        warnings.push(
            "WebUI open_browser is configured but browser launch is not implemented".to_string(),
        );
    }
    warnings
}

fn warning_server(message: impl Into<String>) -> WebServer {
    let message = message.into();
    WebServer {
        status: WebStartupStatus::warning(message.clone(), Utc::now()),
        local_addr: None,
        warnings: vec![message],
        handle: None,
        tunnel: None,
    }
}

fn bind_address(host: &str, port: u16) -> String {
    let trimmed = host.trim();
    if trimmed.starts_with('[') || !trimmed.contains(':') {
        return format!("{trimmed}:{port}");
    }
    format!("[{trimmed}]:{port}")
}

fn web_url(host: &str, port: u16) -> String {
    let trimmed = host.trim();
    if trimmed.starts_with('[') || !trimmed.contains(':') {
        return format!("http://{trimmed}:{port}");
    }
    format!("http://[{trimmed}]:{port}")
}
