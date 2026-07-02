use std::time::Duration;

use ed_sentry::web::start_with_state;

use crate::fake_cloudflared::{fake_cloudflared, FakeCloudflared};
use crate::support::{
    api_runtime, api_store, env_lock, request, write_dist, write_tunnel_api_config,
};

const TUNNEL_PASSWORD: &str = "fixture-tunnel-password";
const FIRST_TUNNEL_HOST: &str = "fixture.trycloudflare.com";
const SECOND_TUNNEL_HOST: &str = "rotated.trycloudflare.com";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_policy_accepts_only_active_tunnel_host_and_rejects_unrelated_trycloudflare_host() {
    // Given: a running fake tunnel with a known public hostname.
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "web policy dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let fake = fake_cloudflared(temp_dir.path(), FakeCloudflared::EmitUrlThenWait);
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    write_tunnel_api_config(&config_path, temp_dir.path(), TUNNEL_PASSWORD);
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    let started = start_tunnel(port);
    assert!(started.starts_with("HTTP/1.1 200 OK"), "{started}");

    // When: the dashboard and unprotected APIs are requested through the active host and an unrelated tunnel-like host.
    let active_dashboard = request(
        port,
        &format!("GET / HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nConnection: close\r\n\r\n"),
    );
    let active_snapshot = request(
        port,
        &format!(
            "GET /api/snapshot HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nConnection: close\r\n\r\n"
        ),
    );
    let unrelated_snapshot = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: attacker.trycloudflare.com\r\nConnection: close\r\n\r\n",
    );

    // Then: only the exact active tunnel host is trusted.
    assert!(
        active_dashboard.starts_with("HTTP/1.1 200 OK"),
        "{active_dashboard}"
    );
    assert!(
        active_dashboard.contains("web policy dist"),
        "{active_dashboard}"
    );
    assert!(
        active_snapshot.starts_with("HTTP/1.1 200 OK"),
        "{active_snapshot}"
    );
    assert!(
        unrelated_snapshot.starts_with("HTTP/1.1 403 Forbidden"),
        "{unrelated_snapshot}"
    );
    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_policy_rejects_stale_tunnel_host_after_crash_and_restart() {
    // Given: a fake Cloudflare process that emits a one-shot first host then a live second host.
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "stale policy dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let counter = temp_dir.path().join("counter");
    let fake = fake_cloudflared(
        temp_dir.path(),
        FakeCloudflared::EmitFirstUrlThenExit(counter),
    );
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    write_tunnel_api_config(&config_path, temp_dir.path(), TUNNEL_PASSWORD);
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    let first = start_tunnel(port);
    assert!(first.starts_with("HTTP/1.1 200 OK"), "{first}");
    tokio::time::sleep(Duration::from_millis(100)).await;

    // When: the crashed first process is refreshed and a replacement tunnel starts.
    let second = start_tunnel(port);
    let stale_snapshot = request(
        port,
        &format!(
            "GET /api/snapshot HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nConnection: close\r\n\r\n"
        ),
    );
    let active_snapshot = request(
        port,
        &format!(
            "GET /api/snapshot HTTP/1.1\r\nHost: {SECOND_TUNNEL_HOST}\r\nConnection: close\r\n\r\n"
        ),
    );

    // Then: stale tunnel hosts are no longer trusted after restart.
    assert!(second.starts_with("HTTP/1.1 200 OK"), "{second}");
    assert!(
        stale_snapshot.starts_with("HTTP/1.1 403 Forbidden"),
        "{stale_snapshot}"
    );
    assert!(
        active_snapshot.starts_with("HTTP/1.1 200 OK"),
        "{active_snapshot}"
    );
    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

fn start_tunnel(port: u16) -> String {
    request(
        port,
        "POST /api/tunnel/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    )
}
