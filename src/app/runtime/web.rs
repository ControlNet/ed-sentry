use chrono::Utc;

use crate::app::{ServiceStatusKind, TunnelProvider, TunnelStatus, WebStartupStatus};
use crate::config::RuntimeConfig;
use crate::text::line_safe;
use crate::web::tunnel_state::WebTunnelState;
use crate::web::WebServer;

use super::MonitorRuntime;

pub async fn start_webui(config: &RuntimeConfig, runtime: &mut MonitorRuntime) -> WebServer {
    let web_server = start_webui_silent(config, runtime).await;
    print_web_startup_status(&web_server.startup_status());
    print_web_warnings(web_server.warnings());
    web_server
}

pub async fn start_webui_silent(config: &RuntimeConfig, runtime: &mut MonitorRuntime) -> WebServer {
    let web_server = crate::web::start_with_state(config, runtime.event_store()).await;
    runtime.web_status = web_server.startup_status();
    web_server
}

pub(crate) async fn start_tunnel_after_webui(
    config: &RuntimeConfig,
    runtime: &mut MonitorRuntime,
    web_server: &WebServer,
    watch_capable: bool,
) -> Option<WebTunnelState> {
    let Some(tunnel) = web_server.tunnel() else {
        let status = TunnelStatus::disabled(TunnelProvider::CloudflareQuick);
        runtime.set_tunnel_status(status);
        return None;
    };
    let status = tunnel
        .apply_startup_policy(config, watch_capable, Utc::now())
        .await;
    runtime.set_tunnel_status(status);
    Some(tunnel)
}

fn print_web_startup_status(status: &WebStartupStatus) {
    match status.kind {
        ServiceStatusKind::Disabled => {
            println!("Info: WebUI disabled - operating with terminal output only\n");
        }
        ServiceStatusKind::Running => {
            let url = status.url.as_deref().unwrap_or("[unavailable]");
            println!("Info: WebUI available at {}\n", line_safe(url));
        }
        ServiceStatusKind::Starting | ServiceStatusKind::Warning | ServiceStatusKind::Error => {
            println!("Info: WebUI unavailable - operating with terminal output only\n");
        }
    }
}

fn print_web_warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("Warning: {}", line_safe(warning));
    }
}
