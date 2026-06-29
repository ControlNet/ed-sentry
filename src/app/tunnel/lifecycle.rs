use chrono::{DateTime, Utc};

use crate::config::RuntimeConfig;
use crate::web::WebServer;

use super::{CloudflareQuickTunnelProvider, TunnelProvider, TunnelStatus};

pub struct TunnelLifecycleManager {
    provider: CloudflareQuickTunnelProvider,
    configured_provider: TunnelProvider,
    bound_port: Option<u16>,
    watch_capable: bool,
}

impl TunnelLifecycleManager {
    pub fn new(
        provider: CloudflareQuickTunnelProvider,
        bound_port: Option<u16>,
        watch_capable: bool,
    ) -> Self {
        Self {
            provider,
            configured_provider: TunnelProvider::CloudflareQuick,
            bound_port,
            watch_capable,
        }
    }

    pub fn for_web_server(web_server: &WebServer, watch_capable: bool) -> Self {
        Self::new(
            CloudflareQuickTunnelProvider::disabled(),
            web_server.bound_port(),
            watch_capable,
        )
    }

    pub async fn apply_startup_policy(
        &mut self,
        config: &RuntimeConfig,
        now: DateTime<Utc>,
    ) -> TunnelStatus {
        self.configure_provider(config);
        match self.configured_provider {
            TunnelProvider::Ssh => TunnelStatus::unsupported(TunnelProvider::Ssh, now),
            TunnelProvider::CloudflareQuick => {
                self.apply_cloudflare_startup_policy(config, now).await
            }
        }
    }

    pub fn configure_provider(&mut self, config: &RuntimeConfig) {
        self.configured_provider = tunnel_provider_from_config(config);
    }

    async fn apply_cloudflare_startup_policy(
        &mut self,
        config: &RuntimeConfig,
        now: DateTime<Utc>,
    ) -> TunnelStatus {
        if !self.watch_capable || !config.web.enabled {
            return TunnelStatus::disabled(TunnelProvider::CloudflareQuick);
        }
        if self.bound_port.is_none() {
            return TunnelStatus::disabled_with_message(
                TunnelProvider::CloudflareQuick,
                "WebUI is unavailable; tunnel cannot start",
                now,
            );
        }
        if config.tunnel.auto_start {
            return self.provider.start_for_port(self.bound_port, now).await;
        }
        self.provider.mark_startable()
    }

    pub async fn manual_start(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        match self.configured_provider {
            TunnelProvider::Ssh => TunnelStatus::unsupported(TunnelProvider::Ssh, now),
            TunnelProvider::CloudflareQuick => {
                self.provider.start_for_port(self.bound_port, now).await
            }
        }
    }

    pub fn refresh(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        self.provider.refresh(now)
    }

    pub fn status(&self) -> TunnelStatus {
        self.provider.current_status()
    }

    pub fn provider(&self) -> &CloudflareQuickTunnelProvider {
        &self.provider
    }
}

fn tunnel_provider_from_config(config: &RuntimeConfig) -> TunnelProvider {
    match config.tunnel.provider.as_str() {
        "ssh" => TunnelProvider::Ssh,
        _ => TunnelProvider::CloudflareQuick,
    }
}
