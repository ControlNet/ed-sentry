use chrono::{DateTime, Utc};
#[cfg(test)]
use tokio::process::Child;

use crate::config::RuntimeConfig;
use crate::web::WebServer;

use super::provider::{AbandonedCloudflareProcess, CloudflareStartStep, CompletedCloudflareStart};
use super::{CloudflareQuickTunnelProvider, TunnelProvider, TunnelStatus, TunnelStatusKind};

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
        match self.prepare_startup_policy(config, now) {
            CloudflareStartStep::Immediate(status) => status,
            CloudflareStartStep::Prepared(prepared) => {
                let completed = CloudflareQuickTunnelProvider::run_prepared_start(prepared).await;
                let completed_at = Utc::now().max(now);
                let (status, abandoned) = self.commit_start(completed, completed_at);
                if let Some(abandoned) = abandoned {
                    CloudflareQuickTunnelProvider::cleanup_abandoned_start(abandoned).await;
                }
                status
            }
        }
    }

    pub(crate) fn prepare_startup_policy(
        &mut self,
        config: &RuntimeConfig,
        now: DateTime<Utc>,
    ) -> CloudflareStartStep {
        self.configured_provider = tunnel_provider_from_config(config);
        match self.configured_provider {
            TunnelProvider::Ssh => {
                let current = self.provider.refresh(now);
                if matches!(
                    current.kind,
                    TunnelStatusKind::Starting | TunnelStatusKind::Running
                ) {
                    return CloudflareStartStep::Immediate(current);
                }
                CloudflareStartStep::Immediate(
                    self.provider
                        .set_status_if_idle(TunnelStatus::unsupported(TunnelProvider::Ssh, now)),
                )
            }
            TunnelProvider::CloudflareQuick => {
                if !self.watch_capable || !config.web.enabled {
                    return CloudflareStartStep::Immediate(self.provider.set_status_if_idle(
                        TunnelStatus::disabled(TunnelProvider::CloudflareQuick),
                    ));
                }
                if self.bound_port.is_none() {
                    return CloudflareStartStep::Immediate(self.provider.set_status_if_idle(
                        TunnelStatus::disabled_with_message(
                            TunnelProvider::CloudflareQuick,
                            "WebUI is unavailable; tunnel cannot start",
                            now,
                        ),
                    ));
                }
                if config.tunnel.auto_start {
                    return self.provider.prepare_start(self.bound_port, now);
                }
                CloudflareStartStep::Immediate(self.provider.mark_startable_at(now))
            }
        }
    }

    pub fn configure_provider(
        &mut self,
        config: &RuntimeConfig,
        now: DateTime<Utc>,
    ) -> TunnelStatus {
        self.configured_provider = tunnel_provider_from_config(config);
        let current = self.provider.refresh(now);
        if matches!(
            current.kind,
            TunnelStatusKind::Starting | TunnelStatusKind::Running
        ) {
            return current;
        }
        match self.configured_provider {
            TunnelProvider::Ssh => self
                .provider
                .set_status_if_idle(TunnelStatus::unsupported(TunnelProvider::Ssh, now)),
            TunnelProvider::CloudflareQuick => {
                if !self.watch_capable {
                    return self.provider.set_status_if_idle(TunnelStatus::disabled_at(
                        TunnelProvider::CloudflareQuick,
                        now,
                    ));
                }
                if self.bound_port.is_none() {
                    return self
                        .provider
                        .set_status_if_idle(TunnelStatus::disabled_with_message(
                            TunnelProvider::CloudflareQuick,
                            "WebUI is unavailable; tunnel cannot start",
                            now,
                        ));
                }
                self.provider.mark_startable_at(now)
            }
        }
    }

    pub async fn manual_start(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        match self.prepare_manual_start(now) {
            CloudflareStartStep::Immediate(status) => status,
            CloudflareStartStep::Prepared(prepared) => {
                let completed = CloudflareQuickTunnelProvider::run_prepared_start(prepared).await;
                let completed_at = Utc::now().max(now);
                let (status, abandoned) = self.commit_start(completed, completed_at);
                if let Some(abandoned) = abandoned {
                    CloudflareQuickTunnelProvider::cleanup_abandoned_start(abandoned).await;
                }
                status
            }
        }
    }

    pub(crate) fn prepare_manual_start(&mut self, now: DateTime<Utc>) -> CloudflareStartStep {
        match self.configured_provider {
            TunnelProvider::Ssh => {
                let current = self.provider.refresh(now);
                if matches!(
                    current.kind,
                    TunnelStatusKind::Starting | TunnelStatusKind::Running
                ) {
                    return CloudflareStartStep::Immediate(current);
                }
                CloudflareStartStep::Immediate(
                    self.provider
                        .set_status_if_idle(TunnelStatus::unsupported(TunnelProvider::Ssh, now)),
                )
            }
            TunnelProvider::CloudflareQuick => self.provider.prepare_start(self.bound_port, now),
        }
    }

    pub(crate) fn commit_start(
        &mut self,
        completed: CompletedCloudflareStart,
        completed_at: DateTime<Utc>,
    ) -> (TunnelStatus, Option<AbandonedCloudflareProcess>) {
        self.provider.commit_start(completed, completed_at)
    }

    #[cfg(test)]
    pub(crate) fn prepare_stop(&mut self, now: DateTime<Utc>) -> (TunnelStatus, Option<Child>) {
        let (status, child) = self.provider.prepare_stop(now);
        if self.configured_provider == TunnelProvider::Ssh {
            let unsupported = self
                .provider
                .set_status_if_idle(TunnelStatus::unsupported(TunnelProvider::Ssh, now));
            return (unsupported, child);
        }
        (status, child)
    }

    pub fn refresh(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        let current = self.provider.refresh(now);
        if self.configured_provider == TunnelProvider::Ssh
            && !matches!(
                current.kind,
                TunnelStatusKind::Starting | TunnelStatusKind::Running
            )
        {
            if current.kind == TunnelStatusKind::Unsupported
                && current.provider == TunnelProvider::Ssh
                && current.checked_at.is_some()
            {
                return current;
            }
            return self
                .provider
                .set_status_if_idle(TunnelStatus::unsupported(TunnelProvider::Ssh, now));
        }
        current
    }

    pub fn status(&self) -> TunnelStatus {
        let current = self.provider.current_status();
        if self.configured_provider == TunnelProvider::Ssh
            && !matches!(
                current.kind,
                TunnelStatusKind::Starting | TunnelStatusKind::Running
            )
        {
            if current.kind == TunnelStatusKind::Unsupported
                && current.provider == TunnelProvider::Ssh
            {
                return current;
            }
            return TunnelStatus::unsupported(TunnelProvider::Ssh, Utc::now());
        }
        current
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
