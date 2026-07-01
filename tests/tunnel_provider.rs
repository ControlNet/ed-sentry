use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use chrono::{TimeZone, Utc};
use ed_sentry::app::{
    cloudflare_trycloudflare_url, CloudflareQuickTunnelProvider, TunnelStatusKind,
};
use tempfile::TempDir;
use tokio::sync::{Mutex, MutexGuard};

static CLOUDFLARED_PROCESS_TEST: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

enum FakeCloudflared {
    StdoutUrlWithArgs(PathBuf),
    StderrUrl,
    DrainingOutput(PathBuf),
    NoUrl,
    ExitBeforeUrl,
    StdoutUrlLongRunning,
}

fn fixture_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
        .single()
        .unwrap()
}

#[test]
fn tunnel_provider_url_parser_selects_first_trycloudflare_url() {
    // Given: cloudflared-style log text with unrelated URLs before the tunnel URL.
    let output = "open http://127.0.0.1 then https://fixture.trycloudflare.com/path";

    // When: the provider parser scans the line.
    let parsed = cloudflare_trycloudflare_url(output);

    // Then: only the first TryCloudflare HTTPS URL is accepted.
    assert_eq!(
        parsed.as_deref(),
        Some("https://fixture.trycloudflare.com/path")
    );
    assert_eq!(cloudflare_trycloudflare_url("https://example.com"), None);
}

#[tokio::test]
async fn tunnel_provider_reads_url_from_stdout_and_uses_required_command_args() {
    let _guard = lock_cloudflared_process_test().await;

    // Given: a fake cloudflared executable that emits the fixture URL on stdout.
    let temp = TempDir::new().unwrap();
    let args_log = temp.path().join("args.log");
    let fake = fake_cloudflared(
        temp.path(),
        FakeCloudflared::StdoutUrlWithArgs(args_log.clone()),
    );
    let mut provider = provider_with_fake(fake);

    // When: the provider starts for a bound local WebUI port.
    let status = provider.start_for_port(Some(8765), fixture_time()).await;

    // Then: it runs with the parsed URL and exact cloudflared tunnel command shape.
    assert_eq!(status.kind, TunnelStatusKind::Running);
    assert_eq!(
        status.public_url.as_deref(),
        Some("https://fixture.trycloudflare.com")
    );
    let args = fs::read_to_string(args_log).unwrap();
    assert_eq!(
        args.lines().collect::<Vec<_>>(),
        [
            "tunnel",
            "--url",
            "http://127.0.0.1:8765",
            "--metrics",
            "127.0.0.1:0"
        ]
    );
}

#[tokio::test]
async fn tunnel_provider_reads_url_from_stderr() {
    let _guard = lock_cloudflared_process_test().await;

    // Given: a fake cloudflared executable that logs the tunnel URL on stderr.
    let temp = TempDir::new().unwrap();
    let fake = fake_cloudflared(temp.path(), FakeCloudflared::StderrUrl);
    let mut provider = provider_with_fake(fake);

    // When: the provider starts.
    let status = provider.start_for_port(Some(8765), fixture_time()).await;

    // Then: stderr is parsed the same as stdout.
    assert_eq!(status.kind, TunnelStatusKind::Running);
    assert_eq!(
        status.public_url.as_deref(),
        Some("https://fixture.trycloudflare.com")
    );
}

#[tokio::test]
async fn tunnel_provider_drains_output_after_url_is_reported() {
    let _guard = lock_cloudflared_process_test().await;

    // Given: real cloudflared emits a URL and then keeps logging connectivity output.
    let temp = TempDir::new().unwrap();
    let drained_marker = temp.path().join("drained.marker");
    let fake = fake_cloudflared(
        temp.path(),
        FakeCloudflared::DrainingOutput(drained_marker.clone()),
    );
    let mut provider = provider_with_fake(fake);

    // When: the provider observes the URL and leaves the child running.
    let status = provider.start_for_port(Some(8765), fixture_time()).await;

    // Then: output readers continue draining after URL detection instead of blocking the child.
    assert_eq!(status.kind, TunnelStatusKind::Running);
    wait_until(Duration::from_secs(2), || drained_marker.exists()).await;
    assert!(drained_marker.exists());
}

#[tokio::test]
async fn tunnel_provider_reports_retryable_error_when_no_url_arrives_before_timeout() {
    let _guard = lock_cloudflared_process_test().await;

    // Given: a fake cloudflared executable that stays alive but never emits a tunnel URL.
    let temp = TempDir::new().unwrap();
    let fake = fake_cloudflared(temp.path(), FakeCloudflared::NoUrl);
    let mut provider = provider_with_fake(fake);

    // When: the URL wait expires.
    let status = provider.start_for_port(Some(8765), fixture_time()).await;

    // Then: startup fails retryably without leaving a trusted active tunnel.
    assert_eq!(status.kind, TunnelStatusKind::Error);
    assert!(status.retryable_error);
    assert!(status.public_url.is_none());
    assert!(provider.active_tunnel().is_none());
}

#[tokio::test]
async fn tunnel_provider_does_not_keep_starting_when_start_future_is_cancelled() {
    let _guard = lock_cloudflared_process_test().await;

    // Given: a fake cloudflared executable that starts but never reports a tunnel URL.
    let temp = TempDir::new().unwrap();
    let fake = fake_cloudflared(temp.path(), FakeCloudflared::NoUrl);
    let mut provider = provider_with_fake(fake);

    // When: the start future is cancelled like a browser refresh aborting /api/tunnel/start.
    let cancelled = tokio::time::timeout(
        Duration::from_millis(100),
        provider.start_for_port(Some(8765), fixture_time()),
    )
    .await;
    assert!(cancelled.is_err());
    let refreshed = provider.refresh(fixture_time());

    // Then: the provider can be retried instead of remaining STARTING forever with no child.
    assert_ne!(refreshed.kind, TunnelStatusKind::Starting);
    assert!(provider.active_tunnel().is_none());
}

#[tokio::test]
async fn tunnel_provider_reports_retryable_error_when_child_exits_before_url() {
    let _guard = lock_cloudflared_process_test().await;

    // Given: a fake cloudflared executable that exits before reporting a URL.
    let temp = TempDir::new().unwrap();
    let fake = fake_cloudflared(temp.path(), FakeCloudflared::ExitBeforeUrl);
    let mut provider = provider_with_fake(fake);

    // When: the provider observes the early process exit.
    let status = provider.start_for_port(Some(8765), fixture_time()).await;

    // Then: the status is retryable error data, not a panic.
    assert_eq!(status.kind, TunnelStatusKind::Error);
    assert!(status.retryable_error);
    assert!(status.message.unwrap().contains("exited before URL"));
}

#[tokio::test]
async fn tunnel_provider_drop_cleans_up_running_child() {
    let _guard = lock_cloudflared_process_test().await;

    // Given: a running fake cloudflared process owned by the provider.
    let temp = TempDir::new().unwrap();
    let fake = fake_cloudflared(temp.path(), FakeCloudflared::StdoutUrlLongRunning);
    let child_id = {
        let mut provider = provider_with_fake(fake);
        let status = provider.start_for_port(Some(8765), fixture_time()).await;
        assert_eq!(status.kind, TunnelStatusKind::Running);
        provider.active_child_id().unwrap()
    };

    // When: the provider is dropped.
    wait_until(Duration::from_secs(2), || !process_is_alive(child_id)).await;

    // Then: the child process no longer remains alive.
    assert!(!process_is_alive(child_id));
}

fn provider_with_fake(fake: PathBuf) -> CloudflareQuickTunnelProvider {
    CloudflareQuickTunnelProvider::with_executable_and_timeout(fake, Duration::from_secs(5))
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
        FakeCloudflared::StdoutUrlWithArgs(path) => format!(
            "printf '%s\\n' \"$@\" >> {}; printf '%s\\n' 'https://fixture.trycloudflare.com'; sleep 30",
            shell_quote(path)
        ),
        FakeCloudflared::StderrUrl => {
            "printf '%s\\n' 'https://fixture.trycloudflare.com' >&2; while :; do sleep 1; done"
                .to_string()
        }
        FakeCloudflared::DrainingOutput(path) => format!(
            "printf '%s\\n' 'https://fixture.trycloudflare.com'; i=0; while [ $i -lt 200000 ]; do printf '%s\\n' 'post-url connectivity precheck log line with enough bytes to fill a pipe'; i=$((i + 1)); done; printf 'drained\\n' > {}; while :; do sleep 1; done",
            shell_quote(path)
        ),
        FakeCloudflared::NoUrl => "sleep 30".to_string(),
        FakeCloudflared::ExitBeforeUrl => "exit 42".to_string(),
        FakeCloudflared::StdoutUrlLongRunning => {
            "printf '%s\\n' 'https://fixture.trycloudflare.com'; sleep 30".to_string()
        }
    };
    format!("#!/bin/sh\n{body}\n")
}

fn fake_cloudflared_batch(behavior: &FakeCloudflared) -> String {
    let body = match behavior {
        FakeCloudflared::StdoutUrlWithArgs(path) => format!(
            ":args\r\nif \"%~1\"==\"\" goto afterargs\r\necho %~1>> {}\r\nshift\r\ngoto args\r\n:afterargs\r\necho https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL",
            batch_quote(path)
        ),
        FakeCloudflared::StderrUrl => {
            "echo https://fixture.trycloudflare.com 1>&2\r\n:loop\r\nping -n 2 127.0.0.1 >NUL\r\ngoto loop".to_string()
        }
        FakeCloudflared::DrainingOutput(path) => format!(
            "echo https://fixture.trycloudflare.com\r\nfor /L %%i in (1,1,2000) do echo post-url connectivity precheck log line with enough bytes to fill a pipe\r\necho drained> {}\r\n:loop\r\nping -n 2 127.0.0.1 >NUL\r\ngoto loop",
            batch_quote(path)
        ),
        FakeCloudflared::NoUrl => "ping -n 31 127.0.0.1 >NUL".to_string(),
        FakeCloudflared::ExitBeforeUrl => "exit /B 42".to_string(),
        FakeCloudflared::StdoutUrlLongRunning => {
            "echo https://fixture.trycloudflare.com\r\nping -n 31 127.0.0.1 >NUL".to_string()
        }
    };
    format!("@echo off\r\n{body}\r\n")
}

fn batch_quote(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

async fn wait_until(timeout: Duration, mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if predicate() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn lock_cloudflared_process_test() -> MutexGuard<'static, ()> {
    CLOUDFLARED_PROCESS_TEST.lock().await
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    Path::new("/proc").join(pid.to_string()).exists()
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    false
}
