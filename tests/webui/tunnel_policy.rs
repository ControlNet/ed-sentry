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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_policy_rejects_cross_site_mutations_and_keeps_same_origin_and_headerless_requests() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "csrf policy dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let fake = fake_cloudflared(temp_dir.path(), FakeCloudflared::EmitUrlThenWait);
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    // Empty password ensures tunnel start never requires a password before starting.
    write_tunnel_api_config(&config_path, temp_dir.path(), "");
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    // Cross-site / untrusted Origin requests to /api/tunnel/start must fail with 403 without starting tunnel.
    let cross_origin_start = request(
        port,
        "POST /api/tunnel/start HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: https://evil.example\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    let cross_site_header_start = request(
        port,
        "POST /api/tunnel/start HTTP/1.1\r\nHost: 127.0.0.1\r\nSec-Fetch-Site: cross-site\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(
        cross_origin_start.starts_with("HTTP/1.1 403 Forbidden"),
        "{cross_origin_start}"
    );
    assert!(
        cross_site_header_start.starts_with("HTTP/1.1 403 Forbidden"),
        "{cross_site_header_start}"
    );

    // Same-origin start without any config_password succeeds.
    let same_origin_start = request(
        port,
        &format!(
            "POST /api/tunnel/start HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nSec-Fetch-Site: same-origin\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        same_origin_start.starts_with("HTTP/1.1 200 OK"),
        "{same_origin_start}"
    );

    // Configure a tunnel password via CLI-style (no Origin / Sec-Fetch-Site) PUT /api/config.
    let set_pw = format!(
        r#"{{"tunnel":{{"provider":"cloudflare_quick","auto_start":false,"config_password_replacement":"{TUNNEL_PASSWORD}","clear_config_password":false}}}}"#
    );
    let cli_put = request(
        port,
        &format!(
            "PUT /api/config HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{set_pw}",
            set_pw.len()
        ),
    );
    assert!(cli_put.starts_with("HTTP/1.1 200 OK"), "{cli_put}");

    // Cross-site login on /api/tunnel/login is rejected with 403 even with the right password.
    let login_body = format!(r#"{{"password":"{TUNNEL_PASSWORD}"}}"#);
    let cross_origin_login = request(
        port,
        &format!(
            "POST /api/tunnel/login HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nOrigin: https://evil.example\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{login_body}",
            login_body.len()
        ),
    );
    let cross_site_login = request(
        port,
        &format!(
            "POST /api/tunnel/login HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nOrigin: https://{FIRST_TUNNEL_HOST}\r\nSec-Fetch-Site: cross-site\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{login_body}",
            login_body.len()
        ),
    );
    assert!(
        cross_origin_login.starts_with("HTTP/1.1 403 Forbidden"),
        "{cross_origin_login}"
    );
    assert!(
        cross_site_login.starts_with("HTTP/1.1 403 Forbidden"),
        "{cross_site_login}"
    );

    // Same-origin tunnel login succeeds and issues a token; cross-site PUT /api/config with valid Bearer still gets 403.
    let same_origin_login = request(
        port,
        &format!(
            "POST /api/tunnel/login HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nOrigin: https://{FIRST_TUNNEL_HOST}\r\nSec-Fetch-Site: same-origin\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{login_body}",
            login_body.len()
        ),
    );
    assert!(
        same_origin_login.starts_with("HTTP/1.1 200 OK"),
        "{same_origin_login}"
    );
    let token = crate::support::json_body(&same_origin_login)["token"]
        .as_str()
        .unwrap()
        .to_string();

    let mutation = r#"{"monitor":{"warn_kill_rate":25}}"#;
    let cross_site_put = request(
        port,
        &format!(
            "PUT /api/config HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nOrigin: https://evil.example\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{mutation}",
            mutation.len()
        ),
    );
    let same_origin_tunnel_put = request(
        port,
        &format!(
            "PUT /api/config HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nOrigin: https://{FIRST_TUNNEL_HOST}\r\nSec-Fetch-Site: same-origin\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{mutation}",
            mutation.len()
        ),
    );
    assert!(
        cross_site_put.starts_with("HTTP/1.1 403 Forbidden"),
        "{cross_site_put}"
    );
    assert!(
        same_origin_tunnel_put.starts_with("HTTP/1.1 200 OK"),
        "{same_origin_tunnel_put}"
    );

    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}
