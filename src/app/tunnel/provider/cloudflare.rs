use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::process::Child;
use tokio::task::JoinHandle;

use crate::text::line_safe;

use super::super::{ActiveTunnel, TunnelProvider, TunnelProviderController};
use super::super::{TunnelSession, TunnelSessionId, TunnelStatus, TunnelStatusKind};

mod parser;
mod process;
mod resolver;

pub use parser::cloudflare_trycloudflare_url;

use parser::public_host;
use process::{abort_tasks, spawn_cloudflared, terminate_child};
use resolver::resolve_cloudflared_executable;

const DEFAULT_URL_TIMEOUT: Duration = Duration::from_secs(10);
const CHILD_STOP_TIMEOUT: Duration = Duration::from_secs(1);

pub struct CloudflareQuickTunnelProvider {
    status: TunnelStatus,
    executable: PathBuf,
    child: Option<Child>,
    readers: Vec<JoinHandle<()>>,
    next_session: u64,
    url_timeout: Duration,
}

impl CloudflareQuickTunnelProvider {
    pub fn disabled() -> Self {
        Self::with_executable(resolve_cloudflared_executable())
    }

    pub fn with_executable(executable: impl Into<PathBuf>) -> Self {
        Self::with_executable_and_timeout(executable, DEFAULT_URL_TIMEOUT)
    }

    pub fn with_executable_and_timeout(
        executable: impl Into<PathBuf>,
        url_timeout: Duration,
    ) -> Self {
        Self {
            status: TunnelStatus::disabled(TunnelProvider::CloudflareQuick),
            executable: executable.into(),
            child: None,
            readers: Vec::new(),
            next_session: 0,
            url_timeout,
        }
    }

    pub fn mark_startable(&mut self) -> TunnelStatus {
        if matches!(
            self.status.kind,
            TunnelStatusKind::Starting | TunnelStatusKind::Running
        ) {
            return self.status.clone();
        }
        self.status = TunnelStatus::manual_start(TunnelProvider::CloudflareQuick);
        self.status.clone()
    }

    pub fn current_status(&self) -> TunnelStatus {
        self.status.clone()
    }

    pub async fn start_for_port(
        &mut self,
        bound_port: Option<u16>,
        now: DateTime<Utc>,
    ) -> TunnelStatus {
        self.refresh(now);
        if matches!(
            self.status.kind,
            TunnelStatusKind::Starting | TunnelStatusKind::Running
        ) {
            return self.status.clone();
        }
        let Some(port) = bound_port else {
            self.status = TunnelStatus::disabled_with_message(
                TunnelProvider::CloudflareQuick,
                "WebUI is not bound to a local port",
                now,
            );
            return self.status.clone();
        };

        self.stop_process().await;
        let session = TunnelSession::starting(self.next_session_id(now), now);
        self.status = TunnelStatus::starting(TunnelProvider::CloudflareQuick, session.clone());
        let local_url = format!("http://127.0.0.1:{port}");

        match self.spawn_and_wait_for_url(&local_url).await {
            Ok((child, readers, public_url)) => {
                self.child = Some(child);
                self.readers = readers;
                self.status = TunnelStatus::running(
                    TunnelProvider::CloudflareQuick,
                    TunnelSession::running(session.id, public_url, now),
                );
            }
            Err(message) => {
                self.status =
                    TunnelStatus::retryable_error(TunnelProvider::CloudflareQuick, message, now);
            }
        }
        self.status.clone()
    }

    pub async fn stop_async(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        self.stop_process().await;
        self.status = TunnelStatus::disabled(TunnelProvider::CloudflareQuick);
        self.status.checked_at = Some(now);
        self.status.clone()
    }

    pub fn refresh(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        let Some(child) = self.child.as_mut() else {
            return self.status.clone();
        };
        match child.try_wait() {
            Ok(Some(exit_status)) => {
                self.child = None;
                self.abort_readers();
                self.status = TunnelStatus::retryable_error(
                    TunnelProvider::CloudflareQuick,
                    format!("cloudflared stopped before tunnel was closed: {exit_status}"),
                    now,
                );
            }
            Ok(None) => {}
            Err(error) => {
                self.child = None;
                self.abort_readers();
                self.status = TunnelStatus::retryable_error(
                    TunnelProvider::CloudflareQuick,
                    format!(
                        "cloudflared status check failed: {}",
                        line_safe(&error.to_string())
                    ),
                    now,
                );
            }
        }
        self.status.clone()
    }

    pub fn active_tunnel(&self) -> Option<ActiveTunnel> {
        if self.status.kind != TunnelStatusKind::Running {
            return None;
        }
        let public_url = self.status.public_url.as_deref()?;
        let session_id = self.status.session_id.clone()?;
        Some(ActiveTunnel::new(public_host(public_url)?, session_id))
    }

    pub fn active_child_id(&self) -> Option<u32> {
        self.child.as_ref().and_then(Child::id)
    }

    async fn spawn_and_wait_for_url(
        &self,
        local_url: &str,
    ) -> Result<(Child, Vec<JoinHandle<()>>, String), String> {
        spawn_cloudflared(&self.executable, local_url, self.url_timeout).await
    }

    async fn stop_process(&mut self) {
        if let Some(child) = self.child.take() {
            terminate_child(child).await;
        }
        self.abort_readers();
    }

    fn stop_process_background(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let _ = tokio::time::timeout(CHILD_STOP_TIMEOUT, child.wait()).await;
                });
            }
        }
        self.abort_readers();
    }

    fn abort_readers(&mut self) {
        abort_tasks(std::mem::take(&mut self.readers));
    }

    fn next_session_id(&mut self, now: DateTime<Utc>) -> TunnelSessionId {
        self.next_session += 1;
        TunnelSessionId::new(format!(
            "cloudflare-{}-{}",
            now.timestamp_millis(),
            self.next_session
        ))
    }
}

impl Drop for CloudflareQuickTunnelProvider {
    fn drop(&mut self) {
        self.stop_process_background();
    }
}

impl TunnelProviderController for CloudflareQuickTunnelProvider {
    fn provider(&self) -> TunnelProvider {
        TunnelProvider::CloudflareQuick
    }

    fn status(&self, _now: DateTime<Utc>) -> TunnelStatus {
        self.status.clone()
    }

    fn start(&mut self, _now: DateTime<Utc>) -> TunnelStatus {
        self.mark_startable()
    }

    fn stop(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        self.stop_process_background();
        self.status = TunnelStatus::disabled(TunnelProvider::CloudflareQuick);
        self.status.checked_at = Some(now);
        self.status.clone()
    }
}
