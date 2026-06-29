use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::app::{
    ActiveTunnel, CloudflareQuickTunnelProvider, TunnelAuth, TunnelAuthError, TunnelAuthIssue,
    TunnelAuthToken, TunnelAuthValidation, TunnelAuthValidationResult, TunnelLifecycleManager,
    TunnelProvider, TunnelStatus,
};
use crate::config::RuntimeConfig;

#[derive(Clone)]
pub(crate) struct WebTunnelState {
    manager: Arc<Mutex<TunnelLifecycleManager>>,
    auth: TunnelAuth,
}

impl WebTunnelState {
    #[cfg(test)]
    pub(crate) fn new(bound_port: Option<u16>) -> Result<Self, TunnelAuthError> {
        Self::with_manager(TunnelLifecycleManager::new(
            CloudflareQuickTunnelProvider::disabled(),
            bound_port,
            true,
        ))
    }

    pub(crate) fn for_config(
        bound_port: Option<u16>,
        config: &RuntimeConfig,
    ) -> Result<Self, TunnelAuthError> {
        let mut manager = TunnelLifecycleManager::new(
            CloudflareQuickTunnelProvider::disabled(),
            bound_port,
            true,
        );
        manager.configure_provider(config);
        Self::with_manager(manager)
    }

    fn with_manager(manager: TunnelLifecycleManager) -> Result<Self, TunnelAuthError> {
        Ok(Self {
            manager: Arc::new(Mutex::new(manager)),
            auth: TunnelAuth::new_per_run()?,
        })
    }

    pub(crate) async fn apply_startup_policy(
        &self,
        config: &RuntimeConfig,
        watch_capable: bool,
        now: DateTime<Utc>,
    ) -> TunnelStatus {
        if !watch_capable {
            return TunnelStatus::disabled(TunnelProvider::CloudflareQuick);
        }
        self.manager
            .lock()
            .await
            .apply_startup_policy(config, now)
            .await
    }

    pub(crate) async fn status(&self, now: DateTime<Utc>) -> TunnelStatus {
        self.manager.lock().await.refresh(now)
    }

    pub(crate) async fn start(&self, now: DateTime<Utc>) -> TunnelStatus {
        self.manager.lock().await.manual_start(now).await
    }

    pub(crate) async fn active_tunnel(&self, now: DateTime<Utc>) -> Option<ActiveTunnel> {
        let mut manager = self.manager.lock().await;
        manager.refresh(now);
        manager.provider().active_tunnel()
    }

    pub(crate) async fn issue_token(
        &self,
        config_password: &str,
        password_attempt: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<TunnelAuthToken>, TunnelAuthError> {
        let Some(active_tunnel) = self.active_tunnel(now).await else {
            return Ok(None);
        };
        self.auth.issue_token(TunnelAuthIssue {
            config_password,
            password_attempt,
            active_tunnel: &active_tunnel,
            now,
        })
    }

    pub(crate) async fn validate_authorization(
        &self,
        config_password: &str,
        authorization_header: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<TunnelAuthValidationResult, TunnelAuthError> {
        let Some(active_tunnel) = self.active_tunnel(now).await else {
            return Err(TunnelAuthError::StaleHost);
        };
        self.auth.validate_authorization(TunnelAuthValidation {
            config_password,
            authorization_header,
            active_tunnel: &active_tunnel,
            now,
        })
    }
}
