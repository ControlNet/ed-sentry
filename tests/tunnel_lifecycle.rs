use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{TimeZone, Utc};
use ed_sentry::app::{
    CloudflareQuickTunnelProvider, TunnelLifecycleManager, TunnelProvider, TunnelStatusKind,
};
use ed_sentry::config::RuntimeConfig;
use tempfile::TempDir;

const FAKE_URL_TIMEOUT: Duration = Duration::from_secs(2);

enum FakeCloudflared {
    LongRunning,
    LogArgsLongRunning(PathBuf),
    LogStartedLongRunning(PathBuf),
    LogStartedThenExit(PathBuf),
    ExitAfterUrl,
}

fn fixture_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
        .single()
        .unwrap()
}

#[tokio::test]
async fn tunnel_lifecycle_auto_start_requires_web_enabled_bound_port_and_watch_capable_runtime() {
    // Given: auto-start is enabled, WebUI is enabled, a bound port exists, and runtime can watch.
    let temp = TempDir::new().unwrap();
    let fake = fake_cloudflared(temp.path(), FakeCloudflared::LongRunning);
    let mut config = RuntimeConfig::default();
    config.web.enabled = true;
    config.tunnel.auto_start = true;
    let provider =
        CloudflareQuickTunnelProvider::with_executable_and_timeout(fake, FAKE_URL_TIMEOUT);
    let mut lifecycle = TunnelLifecycleManager::new(provider, Some(8765), true);

    // When: startup policy is applied.
    let status = lifecycle
        .apply_startup_policy(&config, fixture_time())
        .await;

    // Then: the tunnel starts and exposes the public URL.
    assert_eq!(status.kind, TunnelStatusKind::Running);
    assert_eq!(
        status.public_url.as_deref(),
        Some("https://fixture.trycloudflare.com")
    );
}

#[tokio::test]
async fn tunnel_lifecycle_does_not_auto_start_for_replay_like_non_watch_runtime() {
    // Given: auto-start is enabled but the runtime is not watch-capable.
    let temp = TempDir::new().unwrap();
    let args_log = temp.path().join("args.log");
    let fake = fake_cloudflared(
        temp.path(),
        FakeCloudflared::LogArgsLongRunning(args_log.clone()),
    );
    let mut config = RuntimeConfig::default();
    config.web.enabled = true;
    config.tunnel.auto_start = true;
    let provider =
        CloudflareQuickTunnelProvider::with_executable_and_timeout(fake, FAKE_URL_TIMEOUT);
    let mut lifecycle = TunnelLifecycleManager::new(provider, Some(8765), false);

    // When: startup policy is applied.
    let status = lifecycle
        .apply_startup_policy(&config, fixture_time())
        .await;

    // Then: no child process is spawned.
    assert_eq!(status.kind, TunnelStatusKind::Disabled);
    assert!(!args_log.exists());
}

#[tokio::test]
async fn tunnel_lifecycle_manual_start_keeps_ssh_provider_unsupported() {
    // Given: config selects the future SSH provider and cloudflared would log if invoked.
    let temp = TempDir::new().unwrap();
    let args_log = temp.path().join("args.log");
    let fake = fake_cloudflared(
        temp.path(),
        FakeCloudflared::LogStartedLongRunning(args_log.clone()),
    );
    let mut config = RuntimeConfig::default();
    config.web.enabled = true;
    config.tunnel.provider = "ssh".to_string();
    let provider =
        CloudflareQuickTunnelProvider::with_executable_and_timeout(fake, FAKE_URL_TIMEOUT);
    let mut lifecycle = TunnelLifecycleManager::new(provider, Some(8765), true);
    let startup = lifecycle
        .apply_startup_policy(&config, fixture_time())
        .await;

    // When: manual start is requested after the SSH startup policy result.
    let manual = lifecycle.manual_start(fixture_time()).await;

    // Then: SSH remains unsupported and the Cloudflare executable is never invoked.
    assert_eq!(startup.kind, TunnelStatusKind::Unsupported);
    assert_eq!(startup.provider, TunnelProvider::Ssh);
    assert_eq!(manual.kind, TunnelStatusKind::Unsupported);
    assert_eq!(manual.provider, TunnelProvider::Ssh);
    assert!(!args_log.exists());
}

#[tokio::test]
async fn tunnel_lifecycle_manual_start_is_idempotent_while_running() {
    // Given: a startable lifecycle manager with an invocation log.
    let temp = TempDir::new().unwrap();
    let args_log = temp.path().join("args.log");
    let fake = fake_cloudflared(
        temp.path(),
        FakeCloudflared::LogStartedLongRunning(args_log.clone()),
    );
    let provider =
        CloudflareQuickTunnelProvider::with_executable_and_timeout(fake, FAKE_URL_TIMEOUT);
    let mut lifecycle = TunnelLifecycleManager::new(provider, Some(8765), true);

    // When: manual start is requested twice.
    let first = lifecycle.manual_start(fixture_time()).await;
    let second = lifecycle.manual_start(fixture_time()).await;

    // Then: both calls report the same running session and only one process spawn occurs.
    assert_eq!(first.kind, TunnelStatusKind::Running);
    assert_eq!(second.kind, TunnelStatusKind::Running);
    assert_eq!(first.session_id, second.session_id);
    let starts = fs::read_to_string(args_log).unwrap();
    assert_eq!(starts.lines().collect::<Vec<_>>(), ["started"]);
}

#[tokio::test]
async fn tunnel_lifecycle_no_bound_port_does_not_spawn_for_manual_start() {
    // Given: WebUI did not bind to any local port.
    let temp = TempDir::new().unwrap();
    let args_log = temp.path().join("args.log");
    let fake = fake_cloudflared(
        temp.path(),
        FakeCloudflared::LogStartedThenExit(args_log.clone()),
    );
    let provider =
        CloudflareQuickTunnelProvider::with_executable_and_timeout(fake, FAKE_URL_TIMEOUT);
    let mut lifecycle = TunnelLifecycleManager::new(provider, None, true);

    // When: manual start is requested.
    let status = lifecycle.manual_start(fixture_time()).await;

    // Then: it stays disabled/unavailable and does not spawn cloudflared.
    assert_eq!(status.kind, TunnelStatusKind::Disabled);
    assert!(!args_log.exists());
}

#[tokio::test]
async fn tunnel_lifecycle_clears_active_tunnel_after_crash_and_restart() {
    // Given: the first process reports a URL and then exits.
    let temp = TempDir::new().unwrap();
    let fake = fake_cloudflared(temp.path(), FakeCloudflared::ExitAfterUrl);
    let provider =
        CloudflareQuickTunnelProvider::with_executable_and_timeout(fake, FAKE_URL_TIMEOUT);
    let mut lifecycle = TunnelLifecycleManager::new(provider, Some(8765), true);
    let running = lifecycle.manual_start(fixture_time()).await;
    assert_eq!(running.kind, TunnelStatusKind::Running);
    assert!(lifecycle.provider().active_tunnel().is_some());

    // When: the child exits and status is refreshed.
    tokio::time::sleep(Duration::from_millis(50)).await;
    let crashed = lifecycle.refresh(fixture_time());

    // Then: stale host/session trust is cleared and restart creates a new session.
    assert_eq!(crashed.kind, TunnelStatusKind::Error);
    assert!(lifecycle.provider().active_tunnel().is_none());
    let restarted = lifecycle.manual_start(fixture_time()).await;
    assert_eq!(restarted.kind, TunnelStatusKind::Running);
    assert_ne!(running.session_id, restarted.session_id);
}

fn fake_cloudflared(dir: &Path, behavior: FakeCloudflared) -> PathBuf {
    let path = dir.join(fake_cloudflared_name());
    fs::write(&path, fake_cloudflared_script(&behavior)).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

fn fake_cloudflared_name() -> &'static str {
    if cfg!(windows) {
        "cloudflared-fixture.cmd"
    } else {
        "cloudflared-fixture"
    }
}

fn fake_cloudflared_script(behavior: &FakeCloudflared) -> String {
    if cfg!(windows) {
        return fake_cloudflared_batch(behavior);
    }
    fake_cloudflared_shell(behavior)
}

fn fake_cloudflared_shell(behavior: &FakeCloudflared) -> String {
    let body = match behavior {
        FakeCloudflared::LongRunning => {
            "printf '%s\\n' 'https://fixture.trycloudflare.com'; sleep 30".to_string()
        }
        FakeCloudflared::LogArgsLongRunning(path) => format!(
            "printf '%s\\n' \"$@\" >> {}; printf '%s\\n' 'https://fixture.trycloudflare.com'; while :; do sleep 1; done",
            shell_quote(path)
        ),
        FakeCloudflared::LogStartedLongRunning(path) => format!(
            "printf 'started\\n' >> {}; printf '%s\\n' 'https://fixture.trycloudflare.com'; while :; do sleep 1; done",
            shell_quote(path)
        ),
        FakeCloudflared::LogStartedThenExit(path) => format!(
            "printf 'started\\n' >> {}; printf '%s\\n' 'https://fixture.trycloudflare.com'",
            shell_quote(path)
        ),
        FakeCloudflared::ExitAfterUrl => {
            "printf '%s\\n' 'https://fixture.trycloudflare.com'; exit 0".to_string()
        }
    };
    format!("#!/bin/sh\n{body}\n")
}

fn fake_cloudflared_batch(behavior: &FakeCloudflared) -> String {
    let body = match behavior {
        FakeCloudflared::LongRunning => {
            "echo https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL".to_string()
        }
        FakeCloudflared::LogArgsLongRunning(path) => format!(
            "echo %*>> {}\r\necho https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL",
            batch_quote(path)
        ),
        FakeCloudflared::LogStartedLongRunning(path) => format!(
            "echo started>> {}\r\necho https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL",
            batch_quote(path)
        ),
        FakeCloudflared::LogStartedThenExit(path) => format!(
            "echo started>> {}\r\necho https://fixture.trycloudflare.com",
            batch_quote(path)
        ),
        FakeCloudflared::ExitAfterUrl => "echo https://fixture.trycloudflare.com".to_string(),
    };
    format!("@echo off\r\n{body}\r\n")
}

fn batch_quote(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}
