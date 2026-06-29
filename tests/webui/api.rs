use ed_sentry::config::{AppConfig, ConfigPath, ConfigSource};
use ed_sentry::web::start_with_state;
use serde_json::Value;

use crate::support::{
    api_runtime, api_runtime_for_source, api_store, env_lock, json_body, put_config,
    write_api_config, write_dist,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_config_update_preserves_and_replaces_matrix_token() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, temp_dir.path());
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    let preserve = r#"{"matrix":{"enabled":true,"homeserver":"https://matrix.invalid","room_id":"!room:matrix.invalid","mention_user_id":null,"status_update_interval_seconds":90,"access_token_replacement":null,"clear_access_token":false}}"#;
    let replace = r#"{"matrix":{"enabled":true,"homeserver":"https://matrix.invalid","room_id":"!room:matrix.invalid","mention_user_id":null,"status_update_interval_seconds":90,"access_token_replacement":"replacement-token","clear_access_token":false}}"#;

    let preserve_response = put_config(port, preserve, "127.0.0.1", "http://127.0.0.1:3000");
    let preserved = AppConfig::load_from_path(&config_path).unwrap().config;
    let replace_response = put_config(port, replace, "127.0.0.1", "http://127.0.0.1:3000");
    let replaced = AppConfig::load_from_path(&config_path).unwrap().config;

    assert!(
        preserve_response.starts_with("HTTP/1.1 200 OK"),
        "{preserve_response}"
    );
    assert_eq!(
        preserved.matrix.unwrap().access_token.as_deref(),
        Some("fixture-access-token")
    );
    assert!(
        replace_response.starts_with("HTTP/1.1 200 OK"),
        "{replace_response}"
    );
    assert_eq!(
        replaced.matrix.unwrap().access_token.as_deref(),
        Some("replacement-token")
    );
    assert!(
        !replace_response.contains("replacement-token"),
        "{replace_response}"
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_config_update_clears_matrix_token_only_when_explicit() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, temp_dir.path());
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    let clear = r#"{"matrix":{"enabled":true,"homeserver":"https://matrix.invalid","room_id":"!room:matrix.invalid","mention_user_id":null,"status_update_interval_seconds":60,"access_token_replacement":null,"clear_access_token":true}}"#;

    let clear_response = put_config(port, clear, "127.0.0.1", "http://127.0.0.1:3000");
    let cleared = AppConfig::load_from_path(&config_path).unwrap().config;

    assert!(
        clear_response.starts_with("HTTP/1.1 200 OK"),
        "{clear_response}"
    );
    assert_eq!(cleared.matrix.unwrap().access_token, None);
    assert_eq!(
        json_body(&clear_response)["config"]["matrix"]["access_token_present"],
        Value::Bool(false)
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_config_writes_ignore_origin_but_keep_host_validation() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, temp_dir.path());
    let loopback = api_runtime(&config_path, 0, "127.0.0.1");
    let loopback_server = start_with_state(&loopback, api_store(&loopback)).await;
    let body = r#"{"web":{"enabled":true,"host":"0.0.0.0","port":8765,"open_browser":false}}"#;
    let bad_host = put_config(
        loopback_server.bound_port().unwrap(),
        body,
        "evil.invalid",
        "",
    );
    let remote_origin = put_config(
        loopback_server.bound_port().unwrap(),
        body,
        "127.0.0.1",
        "http://evil.invalid",
    );
    let remote = api_runtime(&config_path, 0, "0.0.0.0");
    let remote_server = start_with_state(&remote, api_store(&remote)).await;
    let remote_response = put_config(remote_server.bound_port().unwrap(), body, "0.0.0.0", "");

    assert!(bad_host.starts_with("HTTP/1.1 403 Forbidden"), "{bad_host}");
    assert!(
        remote_origin.starts_with("HTTP/1.1 200 OK"),
        "{remote_origin}"
    );
    assert!(
        remote_response.starts_with("HTTP/1.1 200 OK"),
        "{remote_response}"
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_allows_remote_bind_snapshot_config_read_and_write() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, temp_dir.path());
    let remote = api_runtime(&config_path, 0, "0.0.0.0");
    let server = start_with_state(&remote, api_store(&remote)).await;
    let port = server.bound_port().unwrap();
    let body = r#"{"web":{"enabled":true,"host":"127.0.0.1","port":8765,"open_browser":false}}"#;

    let snapshot = crate::support::request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: 192.168.50.10\r\nConnection: close\r\n\r\n",
    );
    let config_read = crate::support::request(
        port,
        "GET /api/config HTTP/1.1\r\nHost: 192.168.50.10\r\nConnection: close\r\n\r\n",
    );
    let write_response = put_config(port, body, "192.168.50.10", "http://127.0.0.1:3000");

    assert!(snapshot.starts_with("HTTP/1.1 200 OK"), "{snapshot}");
    assert!(config_read.starts_with("HTTP/1.1 200 OK"), "{config_read}");
    assert_eq!(json_body(&config_read)["policy"]["remote_bind"], true);
    assert_eq!(
        json_body(&config_read)["policy"]["state_changing_enabled"],
        true
    );
    assert!(
        write_response.starts_with("HTTP/1.1 200 OK"),
        "{write_response}"
    );
    assert_eq!(
        json_body(&write_response)["config"]["web"]["host"],
        "127.0.0.1"
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_config_update_malformed_toml_error_is_frontend_safe() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("malformed-config.toml");
    let malformed_original = "[monitor\n";
    std::fs::write(&config_path, malformed_original).unwrap();
    let runtime = api_runtime_for_source(
        ConfigSource::Explicit(ConfigPath::editable(config_path.clone())),
        0,
        "127.0.0.1",
    );
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let body = r#"{"web":{"enabled":true,"host":"127.0.0.1","port":8765,"open_browser":false}}"#;

    let response = put_config(
        server.bound_port().unwrap(),
        body,
        "127.0.0.1",
        "http://127.0.0.1:3000",
    );

    let response_json = json_body(&response);
    println!("malformed_config_response_body={response_json}");
    let raw_config_path = config_path.to_string_lossy();
    let raw_temp_root = temp_dir.path().to_string_lossy();
    assert!(
        response.starts_with("HTTP/1.1 422 Unprocessable Entity"),
        "{response}"
    );
    assert_eq!(response_json["error"]["code"], "malformed_config");
    assert_eq!(
        response_json["error"]["message"],
        "The config file is malformed. Fix the file before saving from the WebUI."
    );
    assert!(!response.contains(raw_config_path.as_ref()), "{response}");
    assert!(!response.contains(raw_temp_root.as_ref()), "{response}");
    assert_eq!(
        std::fs::read_to_string(&config_path).unwrap(),
        malformed_original
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_config_update_write_failure_error_is_frontend_safe_without_partial_write() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let not_a_directory = temp_dir.path().join("not-a-directory");
    std::fs::write(&not_a_directory, "blocks first-save parent").unwrap();
    let target = not_a_directory.join("config.toml");
    let runtime = api_runtime_for_source(
        ConfigSource::Defaults {
            first_save_target: ConfigPath::first_save(target.clone()),
        },
        0,
        "127.0.0.1",
    );
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let body = r#"{"web":{"enabled":true,"host":"127.0.0.1","port":8765,"open_browser":false}}"#;

    let response = put_config(
        server.bound_port().unwrap(),
        body,
        "127.0.0.1",
        "http://127.0.0.1:3000",
    );

    let response_json = json_body(&response);
    println!("config_write_failed_response_body={response_json}");
    let raw_target = target.to_string_lossy();
    let raw_temp_root = temp_dir.path().to_string_lossy();
    assert!(
        response.starts_with("HTTP/1.1 500 Internal Server Error"),
        "{response}"
    );
    assert_eq!(response_json["error"]["code"], "config_write_failed");
    assert_eq!(
        response_json["error"]["message"],
        "The config file could not be saved. Check file permissions and try again."
    );
    assert!(!response.contains(raw_target.as_ref()), "{response}");
    assert!(!response.contains(raw_temp_root.as_ref()), "{response}");
    assert!(!target.exists());
    assert_eq!(
        std::fs::read_to_string(&not_a_directory).unwrap(),
        "blocks first-save parent"
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}
