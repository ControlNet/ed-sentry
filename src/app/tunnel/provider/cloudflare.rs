use std::path::PathBuf;
use std::sync::{Arc, Weak};
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::process::Child;
use tokio::sync::oneshot;
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

pub(crate) enum CloudflareStartStep {
    Immediate(TunnelStatus),
    Prepared(PreparedCloudflareStart),
}

pub(crate) struct PreparedCloudflareStart {
    executable: PathBuf,
    local_url: String,
    url_timeout: Duration,
    session_id: TunnelSessionId,
    generation: u64,
    previous_child: Option<Child>,
    _in_flight_guard: Arc<()>,
    cancel_rx: oneshot::Receiver<()>,
}

pub(crate) struct AbandonedCloudflareProcess {
    child: Child,
    readers: Vec<JoinHandle<()>>,
}

type SpawnCloudflareOutcome = Result<(Child, Vec<JoinHandle<()>>, String), String>;

pub(crate) struct CompletedCloudflareStart {
    session_id: TunnelSessionId,
    generation: u64,
    outcome: Option<SpawnCloudflareOutcome>,
    _in_flight_guard: Arc<()>,
}

pub struct CloudflareQuickTunnelProvider {
    status: TunnelStatus,
    executable: PathBuf,
    child: Option<Child>,
    readers: Vec<JoinHandle<()>>,
    next_session: u64,
    url_timeout: Duration,
    start_generation: u64,
    starting_guard: Option<Weak<()>>,
    start_cancel: Option<oneshot::Sender<()>>,
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
            start_generation: 0,
            starting_guard: None,
            start_cancel: None,
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

    pub fn mark_startable_at(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        if matches!(
            self.status.kind,
            TunnelStatusKind::Starting | TunnelStatusKind::Running
        ) {
            return self.status.clone();
        }
        self.status = TunnelStatus::manual_start_at(TunnelProvider::CloudflareQuick, now);
        self.status.clone()
    }

    pub(crate) fn set_status_if_idle(&mut self, status: TunnelStatus) -> TunnelStatus {
        if matches!(
            self.status.kind,
            TunnelStatusKind::Starting | TunnelStatusKind::Running
        ) {
            return self.status.clone();
        }
        self.status = status;
        self.status.clone()
    }

    pub fn current_status(&self) -> TunnelStatus {
        self.status.clone()
    }

    pub(crate) fn prepare_start(
        &mut self,
        bound_port: Option<u16>,
        now: DateTime<Utc>,
    ) -> CloudflareStartStep {
        self.refresh(now);
        if matches!(
            self.status.kind,
            TunnelStatusKind::Starting | TunnelStatusKind::Running
        ) {
            return CloudflareStartStep::Immediate(self.status.clone());
        }
        let Some(port) = bound_port else {
            self.status = TunnelStatus::disabled_with_message(
                TunnelProvider::CloudflareQuick,
                "WebUI is not bound to a local port",
                now,
            );
            return CloudflareStartStep::Immediate(self.status.clone());
        };

        let previous_child = self.child.take();
        self.abort_readers();
        self.start_generation = self.start_generation.wrapping_add(1);
        let generation = self.start_generation;
        let in_flight_guard = Arc::new(());
        self.starting_guard = Some(Arc::downgrade(&in_flight_guard));
        let (cancel_tx, cancel_rx) = oneshot::channel();
        self.start_cancel = Some(cancel_tx);

        let session = TunnelSession::starting(self.next_session_id(now), now);
        self.status = TunnelStatus::starting(TunnelProvider::CloudflareQuick, session.clone());
        let local_url = format!("http://127.0.0.1:{port}");

        CloudflareStartStep::Prepared(PreparedCloudflareStart {
            executable: self.executable.clone(),
            local_url,
            url_timeout: self.url_timeout,
            session_id: session.id,
            generation,
            previous_child,
            _in_flight_guard: in_flight_guard,
            cancel_rx,
        })
    }

    pub(crate) async fn run_prepared_start(
        mut prepared: PreparedCloudflareStart,
    ) -> CompletedCloudflareStart {
        if let Some(previous_child) = prepared.previous_child.take() {
            terminate_child(previous_child).await;
        }
        let outcome = tokio::select! {
            result = spawn_cloudflared(
                &prepared.executable,
                &prepared.local_url,
                prepared.url_timeout,
            ) => Some(result),
            _ = &mut prepared.cancel_rx => None,
        };
        CompletedCloudflareStart {
            session_id: prepared.session_id,
            generation: prepared.generation,
            outcome,
            _in_flight_guard: prepared._in_flight_guard,
        }
    }

    pub(crate) fn commit_start(
        &mut self,
        completed: CompletedCloudflareStart,
        completed_at: DateTime<Utc>,
    ) -> (TunnelStatus, Option<AbandonedCloudflareProcess>) {
        if self.start_generation != completed.generation {
            let abandoned = match completed.outcome {
                Some(Ok((child, readers, _public_url))) => {
                    Some(AbandonedCloudflareProcess { child, readers })
                }
                _ => None,
            };
            return (self.status.clone(), abandoned);
        }
        self.starting_guard = None;
        self.start_cancel = None;
        match completed.outcome {
            Some(Ok((child, readers, public_url))) => {
                self.child = Some(child);
                self.readers = readers;
                self.status = TunnelStatus::running(
                    TunnelProvider::CloudflareQuick,
                    TunnelSession::running(completed.session_id, public_url, completed_at),
                );
            }
            Some(Err(message)) => {
                self.status = TunnelStatus::retryable_error(
                    TunnelProvider::CloudflareQuick,
                    message,
                    completed_at,
                );
            }
            None => {
                self.status =
                    TunnelStatus::disabled_at(TunnelProvider::CloudflareQuick, completed_at);
            }
        }
        (self.status.clone(), None)
    }

    pub(crate) async fn cleanup_abandoned_start(abandoned: AbandonedCloudflareProcess) {
        terminate_child(abandoned.child).await;
        abort_tasks(abandoned.readers);
    }

    pub(crate) fn prepare_stop(&mut self, now: DateTime<Utc>) -> (TunnelStatus, Option<Child>) {
        self.cancel_in_flight_start();
        let child = self.child.take();
        self.abort_readers();
        self.status = TunnelStatus::disabled_at(TunnelProvider::CloudflareQuick, now);
        (self.status.clone(), child)
    }

    pub(crate) async fn finish_stop(child: Option<Child>) {
        if let Some(child) = child {
            terminate_child(child).await;
        }
    }

    pub async fn start_for_port(
        &mut self,
        bound_port: Option<u16>,
        now: DateTime<Utc>,
    ) -> TunnelStatus {
        match self.prepare_start(bound_port, now) {
            CloudflareStartStep::Immediate(status) => status,
            CloudflareStartStep::Prepared(prepared) => {
                let completed = Self::run_prepared_start(prepared).await;
                let completed_at = Utc::now().max(now);
                let (status, abandoned) = self.commit_start(completed, completed_at);
                if let Some(abandoned) = abandoned {
                    Self::cleanup_abandoned_start(abandoned).await;
                }
                status
            }
        }
    }

    pub async fn stop_async(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        let (status, child) = self.prepare_stop(now);
        Self::finish_stop(child).await;
        status
    }

    pub fn refresh(&mut self, now: DateTime<Utc>) -> TunnelStatus {
        let Some(child) = self.child.as_mut() else {
            if self.status.kind == TunnelStatusKind::Starting {
                let still_starting = self
                    .starting_guard
                    .as_ref()
                    .is_some_and(|guard| guard.strong_count() > 0);
                if !still_starting {
                    self.starting_guard = None;
                    self.start_cancel = None;
                    self.status = TunnelStatus::retryable_error(
                        TunnelProvider::CloudflareQuick,
                        "cloudflared start was interrupted before URL was reported",
                        now,
                    );
                }
            }
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

    fn cancel_in_flight_start(&mut self) {
        self.start_generation = self.start_generation.wrapping_add(1);
        self.starting_guard = None;
        if let Some(cancel) = self.start_cancel.take() {
            let _ = cancel.send(());
        }
    }

    fn stop_process_background(&mut self) {
        self.cancel_in_flight_start();
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
        self.status = TunnelStatus::disabled_at(TunnelProvider::CloudflareQuick, now);
        self.status.clone()
    }
}
