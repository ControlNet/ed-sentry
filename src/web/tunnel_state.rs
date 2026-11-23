use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::app::{
    ActiveTunnel, CloudflareQuickTunnelProvider, CloudflareStartStep, TunnelAuth, TunnelAuthError,
    TunnelAuthIssue, TunnelAuthToken, TunnelAuthValidation, TunnelAuthValidationResult,
    TunnelLifecycleManager, TunnelProvider, TunnelStatus,
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
        manager.configure_provider(config, Utc::now());
        Self::with_manager(manager)
    }

    #[cfg(test)]
    pub(crate) fn from_manager(manager: TunnelLifecycleManager) -> Result<Self, TunnelAuthError> {
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
        let step = {
            let mut manager = self.manager.lock().await;
            manager.prepare_startup_policy(config, now)
        };
        self.finish_start_step(step, now).await
    }

    pub(crate) async fn status(&self, now: DateTime<Utc>) -> TunnelStatus {
        self.manager.lock().await.refresh(now)
    }

    pub(crate) async fn configure_provider(
        &self,
        config: &RuntimeConfig,
        now: DateTime<Utc>,
    ) -> TunnelStatus {
        self.manager.lock().await.configure_provider(config, now)
    }

    pub(crate) async fn start(&self, now: DateTime<Utc>) -> TunnelStatus {
        let step = {
            let mut manager = self.manager.lock().await;
            manager.prepare_manual_start(now)
        };
        self.finish_start_step(step, now).await
    }

    #[cfg(test)]
    pub(crate) async fn stop(&self, now: DateTime<Utc>) -> TunnelStatus {
        let (status, child) = {
            let mut manager = self.manager.lock().await;
            manager.prepare_stop(now)
        };
        CloudflareQuickTunnelProvider::finish_stop(child).await;
        status
    }

    async fn finish_start_step(
        &self,
        step: CloudflareStartStep,
        started_at: DateTime<Utc>,
    ) -> TunnelStatus {
        match step {
            CloudflareStartStep::Immediate(status) => status,
            CloudflareStartStep::Prepared(prepared) => {
                let completed = CloudflareQuickTunnelProvider::run_prepared_start(*prepared).await;
                let completed_at = Utc::now().max(started_at);
                let (status, abandoned) = {
                    let mut manager = self.manager.lock().await;
                    manager.commit_start(completed, completed_at)
                };
                if let Some(abandoned) = abandoned {
                    CloudflareQuickTunnelProvider::cleanup_abandoned_start(abandoned).await;
                }
                status
            }
        }
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    use chrono::Utc;

    use crate::app::{CloudflareQuickTunnelProvider, TunnelLifecycleManager, TunnelStatusKind};

    use super::WebTunnelState;

    #[tokio::test]
    async fn web_tunnel_state_reads_do_not_block_while_start_waits_for_url() {
        let temp_dir = tempfile::tempdir().unwrap();
        let release_signal = temp_dir.path().join("release-url");
        let start_log = temp_dir.path().join("starts.log");
        let fake_bin =
            fake_slow_cloudflared(temp_dir.path(), &release_signal, Some(&start_log), None);
        let provider = CloudflareQuickTunnelProvider::with_executable_and_timeout(
            fake_bin,
            Duration::from_secs(5),
        );
        let state =
            WebTunnelState::from_manager(TunnelLifecycleManager::new(provider, Some(8765), true))
                .unwrap();

        let started_at = Utc::now();
        let start_state = state.clone();
        let start_task = tokio::spawn(async move { start_state.start(started_at).await });

        // Wait until start has marked the provider Starting.
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let status = state.status(Utc::now()).await;
                if status.kind == TunnelStatusKind::Starting {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("start should enter Starting promptly");

        // Readers (status + active_tunnel) must return immediately without blocking on cloudflared.
        let read_start = Instant::now();
        let status_during_start =
            tokio::time::timeout(Duration::from_millis(250), state.status(Utc::now()))
                .await
                .expect("status() must not block while cloudflared is starting");
        let active_during_start =
            tokio::time::timeout(Duration::from_millis(250), state.active_tunnel(Utc::now()))
                .await
                .expect("active_tunnel() must not block while cloudflared is starting");
        assert!(read_start.elapsed() < Duration::from_millis(250));
        assert_eq!(status_during_start.kind, TunnelStatusKind::Starting);
        assert!(active_during_start.is_none());

        // A concurrent start request must not spawn a second cloudflared process.
        let concurrent = tokio::time::timeout(Duration::from_millis(250), state.start(Utc::now()))
            .await
            .expect("concurrent start must return immediately while Starting");
        assert_eq!(concurrent.kind, TunnelStatusKind::Starting);

        // Release the fake cloudflared after a measurable pause to verify completion-time checked_at.
        tokio::time::sleep(Duration::from_millis(30)).await;
        let before_release = Utc::now();
        fs::write(&release_signal, "go").unwrap();

        let final_status = tokio::time::timeout(Duration::from_secs(10), start_task)
            .await
            .expect("start task should finish after release")
            .unwrap();
        assert_eq!(final_status.kind, TunnelStatusKind::Running);
        assert_eq!(
            final_status.public_url.as_deref(),
            Some("https://fixture.trycloudflare.com")
        );
        assert!(
            final_status.checked_at.unwrap() >= before_release,
            "checked_at should reflect completion time"
        );
        let starts = fs::read_to_string(&start_log).unwrap();
        assert_eq!(starts.lines().count(), 1);
    }

    #[tokio::test]
    async fn web_tunnel_state_stop_during_startup_cancels_without_orphan_process() {
        let temp_dir = tempfile::tempdir().unwrap();
        let release_signal = temp_dir.path().join("never-release");
        let pid_file = temp_dir.path().join("child.pid");
        let fake_bin =
            fake_slow_cloudflared(temp_dir.path(), &release_signal, None, Some(&pid_file));
        let provider = CloudflareQuickTunnelProvider::with_executable_and_timeout(
            fake_bin,
            Duration::from_secs(5),
        );
        let state =
            WebTunnelState::from_manager(TunnelLifecycleManager::new(provider, Some(8765), true))
                .unwrap();

        let start_state = state.clone();
        let start_task = tokio::spawn(async move { start_state.start(Utc::now()).await });

        // Wait until the fake cloudflared process is spawned and has written its PID.
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if fs::read_to_string(&pid_file)
                    .ok()
                    .is_some_and(|raw| raw.trim().parse::<u32>().is_ok())
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("fake cloudflared should write its PID");

        let child_pid: u32 = fs::read_to_string(&pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();

        let stopped = tokio::time::timeout(Duration::from_millis(500), state.stop(Utc::now()))
            .await
            .expect("stop() must not block on slow cloudflared startup");
        assert_eq!(stopped.kind, TunnelStatusKind::Disabled);

        let start_result = tokio::time::timeout(Duration::from_secs(5), start_task)
            .await
            .expect("start task should exit promptly after stop")
            .unwrap();
        assert_eq!(start_result.kind, TunnelStatusKind::Disabled);
        assert!(state.active_tunnel(Utc::now()).await.is_none());

        #[cfg(unix)]
        {
            tokio::time::timeout(Duration::from_secs(2), async {
                loop {
                    if !Path::new("/proc").join(child_pid.to_string()).exists() {
                        return;
                    }
                    tokio::time::sleep(Duration::from_millis(15)).await;
                }
            })
            .await
            .expect("stopped in-flight child process must not remain alive");
        }
        #[cfg(not(unix))]
        {
            let _ = child_pid;
        }
    }

    fn fake_slow_cloudflared(
        dir: &Path,
        release_signal: &Path,
        start_log: Option<&Path>,
        pid_file: Option<&Path>,
    ) -> PathBuf {
        let name = if cfg!(windows) {
            "cloudflared-slow.cmd"
        } else {
            "cloudflared-slow"
        };
        let path = dir.join(name);
        let script = if cfg!(windows) {
            let mut lines = vec!["@echo off".to_string()];
            if let Some(log) = start_log {
                lines.push(format!("echo started>> \"{}\"", log.display()));
            }
            if let Some(pid) = pid_file {
                lines.push(format!("(echo 1)> \"{}\"", pid.display()));
            }
            lines.push(":wait".to_string());
            lines.push(format!(
                "if not exist \"{}\" (ping -n 2 127.0.0.1 >NUL\r\ngoto wait)",
                release_signal.display()
            ));
            lines.push("echo https://fixture.trycloudflare.com".to_string());
            lines.push("ping -n 31 127.0.0.1 >NUL".to_string());
            lines.join("\r\n")
        } else {
            let mut parts = vec!["#!/bin/sh".to_string()];
            if let Some(log) = start_log {
                parts.push(format!("printf 'started\\n' >> {}", shell_quote(log)));
            }
            if let Some(pid) = pid_file {
                parts.push(format!("printf '%s\\n' \"$$\" > {}", shell_quote(pid)));
            }
            parts.push(format!(
                "while [ ! -f {} ]; do sleep 0.01; done",
                shell_quote(release_signal)
            ));
            parts.push("printf '%s\\n' 'https://fixture.trycloudflare.com'".to_string());
            parts.push("while :; do sleep 1; done".to_string());
            parts.join("\n")
        };
        fs::write(&path, format!("{script}\n")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    fn shell_quote(path: &Path) -> String {
        format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
    }
}
