use std::fs;
use std::path::Path;

use ed_sentry::config::AppConfig;
use ed_sentry::web::start_with_state;
use serde_json::Value;

use crate::fake_cloudflared::{fake_cloudflared, FakeCloudflared};
use crate::support::{
    api_runtime, api_store, env_lock, json_body, put_config, put_config_with_auth, request,
    write_dist, write_tunnel_api_config,
};

const TUNNEL_PASSWORD: &str = "fixture-tunnel-password";
const FIRST_TUNNEL_HOST: &str = "fixture.trycloudflare.com";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tunnel_routes_status_start_login_success_and_failure() {
    // Given: a WebUI server with a fake Cloudflare process and a non-empty tunnel password.
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "tunnel routes dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let fake = fake_cloudflared(temp_dir.path(), FakeCloudflared::EmitUrlThenWait);
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    write_tunnel_api_config(&config_path, temp_dir.path(), TUNNEL_PASSWORD);
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    // When: status is read, the tunnel is started, and login is attempted twice.
    let initial = request(
        port,
        "GET /api/tunnel/status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    let started = request(
        port,
        "POST /api/tunnel/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    let failed_login = login(port, "127.0.0.1", "wrong-password");
    let successful_login = login(port, "127.0.0.1", TUNNEL_PASSWORD);

    // Then: the exact route set starts the fake tunnel and only the correct password mints a token.
    assert!(initial.starts_with("HTTP/1.1 200 OK"), "{initial}");
    assert!(started.starts_with("HTTP/1.1 200 OK"), "{started}");
    assert_eq!(
        json_body(&started)["kind"],
        Value::String("running".to_string())
    );
    assert_eq!(
        json_body(&started)["public_url"],
        Value::String("https://fixture.trycloudflare.com".to_string())
    );
    assert!(
        failed_login.starts_with("HTTP/1.1 403 Forbidden"),
        "{failed_login}"
    );
    assert!(
        successful_login.starts_with("HTTP/1.1 200 OK"),
        "login failed"
    );
    assert!(json_body(&successful_login)["token"].as_str().is_some());
    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_api_requires_bearer_for_tunnel_host_and_preserves_loopback_access() {
    // Given: a running fake tunnel with a non-empty tunnel config password.
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "config policy dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let fake = fake_cloudflared(temp_dir.path(), FakeCloudflared::EmitUrlThenWait);
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    write_tunnel_api_config(&config_path, temp_dir.path(), TUNNEL_PASSWORD);
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    assert!(start_tunnel(port).starts_with("HTTP/1.1 200 OK"));
    let mutation = r#"{"tunnel":{"provider":"cloudflare_quick","auto_start":true,"config_password_replacement":null,"clear_config_password":false}}"#;

    // When: config APIs are called through the tunnel without and with a valid token, and locally.
    let missing_token_read = request(
        port,
        &format!(
            "GET /api/config HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nConnection: close\r\n\r\n"
        ),
    );
    let token = json_body(&login(port, FIRST_TUNNEL_HOST, TUNNEL_PASSWORD))["token"]
        .as_str()
        .unwrap()
        .to_string();
    let authorized_read = request(
        port,
        &format!(
            "GET /api/config HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    );
    let authorized_write = put_config_with_auth(port, mutation, FIRST_TUNNEL_HOST, &token);
    let unrelated_write =
        put_config_with_auth(port, mutation, "attacker.trycloudflare.com", &token);
    let local_write = put_config(port, mutation, "127.0.0.1", "http://127.0.0.1:3000");

    // Then: tunnel config access requires the active Host plus Bearer auth, while loopback behavior is unchanged.
    assert!(
        missing_token_read.starts_with("HTTP/1.1 403 Forbidden"),
        "{missing_token_read}"
    );
    assert!(
        authorized_read.starts_with("HTTP/1.1 200 OK"),
        "{authorized_read}"
    );
    assert!(
        authorized_write.starts_with("HTTP/1.1 200 OK"),
        "{authorized_write}"
    );
    assert!(
        unrelated_write.starts_with("HTTP/1.1 403 Forbidden"),
        "{unrelated_write}"
    );
    assert_eq!(
        json_body(&authorized_write)["config"]["tunnel"]["auto_start"],
        Value::Bool(true)
    );
    assert!(local_write.starts_with("HTTP/1.1 200 OK"), "{local_write}");
    assert!(
        AppConfig::load_from_path(&config_path)
            .unwrap()
            .config
            .tunnel
            .auto_start
    );
    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_api_allows_tunnel_host_when_password_is_empty() {
    // Given: a running fake tunnel without a configured tunnel config password.
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "empty password dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let fake = fake_cloudflared(temp_dir.path(), FakeCloudflared::EmitUrlThenWait);
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    write_tunnel_api_config(&config_path, temp_dir.path(), "");
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    assert!(start_tunnel(port).starts_with("HTTP/1.1 200 OK"));
    let mutation = r#"{"tunnel":{"provider":"cloudflare_quick","auto_start":true,"config_password_replacement":null,"clear_config_password":false}}"#;

    // When: config APIs are called through the active tunnel without Authorization.
    let read = request(
        port,
        &format!(
            "GET /api/config HTTP/1.1\r\nHost: {FIRST_TUNNEL_HOST}\r\nConnection: close\r\n\r\n"
        ),
    );
    let write = put_config(port, mutation, FIRST_TUNNEL_HOST, "");

    // Then: empty password mode bypasses tunnel config protection.
    assert!(read.starts_with("HTTP/1.1 200 OK"), "{read}");
    assert!(write.starts_with("HTTP/1.1 200 OK"), "{write}");
    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tunnel_routes_manual_start_keeps_ssh_provider_unsupported() {
    // Given: config selects the future SSH provider and cloudflared would log if invoked.
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "ssh provider dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let args_log = temp_dir.path().join("args.log");
    let fake = fake_cloudflared(
        temp_dir.path(),
        FakeCloudflared::LogStartedThenWait(args_log.clone()),
    );
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    write_ssh_tunnel_api_config(&config_path, temp_dir.path());
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    // When: the manual tunnel start API is called.
    let started = start_tunnel(port);

    // Then: SSH stays unsupported and the Cloudflare executable is never invoked.
    assert!(started.starts_with("HTTP/1.1 200 OK"), "{started}");
    assert_eq!(
        json_body(&started)["kind"],
        Value::String("unsupported".to_string())
    );
    assert_eq!(
        json_body(&started)["provider"],
        Value::String("ssh".to_string())
    );
    assert!(!args_log.exists());
    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn config_api_requires_bearer_for_mixed_case_and_trailing_dot_tunnel_host_on_wildcard_bind() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "wildcard tunnel auth dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let fake = fake_cloudflared(temp_dir.path(), FakeCloudflared::EmitUrlThenWait);
    std::env::set_var("ED_SENTRY_CLOUDFLARED_PATH", &fake);
    let config_path = temp_dir.path().join("config.toml");
    write_tunnel_api_config(&config_path, temp_dir.path(), TUNNEL_PASSWORD);
    let runtime = api_runtime(&config_path, 0, "0.0.0.0");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    assert!(start_tunnel(port).starts_with("HTTP/1.1 200 OK"));

    let uppercase_read = request(
        port,
        "GET /api/config HTTP/1.1\r\nHost: FIXTURE.TRYCLOUDFLARE.COM\r\nConnection: close\r\n\r\n",
    );
    let trailing_dot_read = request(
        port,
        "GET /api/config HTTP/1.1\r\nHost: fixture.trycloudflare.com.\r\nConnection: close\r\n\r\n",
    );
    let lan_read = request(
        port,
        "GET /api/config HTTP/1.1\r\nHost: 192.168.50.10\r\nConnection: close\r\n\r\n",
    );

    assert!(
        uppercase_read.starts_with("HTTP/1.1 403 Forbidden"),
        "{uppercase_read}"
    );
    assert_eq!(
        json_body(&uppercase_read)["error"]["code"],
        Value::String("tunnel_config_auth_required".to_string())
    );
    assert!(
        trailing_dot_read.starts_with("HTTP/1.1 403 Forbidden"),
        "{trailing_dot_read}"
    );
    assert_eq!(
        json_body(&trailing_dot_read)["error"]["code"],
        Value::String("tunnel_config_auth_required".to_string())
    );
    assert!(lan_read.starts_with("HTTP/1.1 200 OK"), "{lan_read}");

    let token = json_body(&login(port, FIRST_TUNNEL_HOST, TUNNEL_PASSWORD))["token"]
        .as_str()
        .unwrap()
        .to_string();
    let authorized_uppercase_read = request(
        port,
        &format!(
            "GET /api/config HTTP/1.1\r\nHost: FIXTURE.TRYCLOUDFLARE.COM\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(
        authorized_uppercase_read.starts_with("HTTP/1.1 200 OK"),
        "{authorized_uppercase_read}"
    );

    std::env::remove_var("ED_SENTRY_CLOUDFLARED_PATH");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_config_provider_switch_from_ssh_to_cloudflare_updates_snapshot_to_startable() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "provider switch dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_ssh_tunnel_api_config(&config_path, temp_dir.path());
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    let initial_snapshot = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(
        initial_snapshot.starts_with("HTTP/1.1 200 OK"),
        "{initial_snapshot}"
    );
    assert_eq!(
        json_body(&initial_snapshot)["tunnel"]["kind"],
        Value::String("unsupported".to_string())
    );

    let switch_body = r#"{"tunnel":{"provider":"cloudflare_quick","auto_start":false,"config_password_replacement":null,"clear_config_password":false}}"#;
    let put_response = put_config(port, switch_body, "127.0.0.1", "http://127.0.0.1:3000");
    assert!(
        put_response.starts_with("HTTP/1.1 200 OK"),
        "{put_response}"
    );
    assert_eq!(
        json_body(&put_response)["restart_required"],
        Value::Bool(false)
    );

    let updated_snapshot = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(
        updated_snapshot.starts_with("HTTP/1.1 200 OK"),
        "{updated_snapshot}"
    );
    assert_eq!(
        json_body(&updated_snapshot)["tunnel"]["kind"],
        Value::String("start".to_string())
    );
    assert_eq!(
        json_body(&updated_snapshot)["tunnel"]["provider"],
        Value::String("cloudflare_quick".to_string())
    );

    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

fn start_tunnel(port: u16) -> String {
    request(
        port,
        "POST /api/tunnel/start HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    )
}

fn login(port: u16, host: &str, password: &str) -> String {
    let body = format!(r#"{{"password":"{password}"}}"#);
    request(
        port,
        &format!(
            "POST /api/tunnel/login HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}

fn write_ssh_tunnel_api_config(path: &Path, journal_dir: &Path) {
    fs::write(
        path,
        format!(
            r#"
            [journal]
            folder = {:?}

            [web]
            enabled = true
            host = "127.0.0.1"
            port = 0
            open_browser = false

            [tunnel]
            provider = "ssh"
            auto_start = false
            "#,
            journal_dir.display().to_string()
        ),
    )
    .unwrap();
}
