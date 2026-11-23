use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use ed_sentry::app::runtime::{ConfiguredJournalSelector, MonitorRuntime};
use ed_sentry::app::{MatrixStartupStatus, WebStartupStatus};
use ed_sentry::config::{AppConfig, ConfigPath, ConfigSource, RuntimeConfig};
use ed_sentry::web::{start_with_applier, start_with_state, RuntimeConfigApplier};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::support::{
    api_runtime, api_runtime_for_source, api_store, env_lock, json_body, put_config, request,
    write_api_config, write_dist, write_journal_fixture,
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
async fn webui_api_config_writes_reject_untrusted_origin_and_keep_same_origin_and_cli() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, temp_dir.path());
    let loopback = api_runtime(&config_path, 0, "127.0.0.1");
    let loopback_server = start_with_state(&loopback, api_store(&loopback)).await;
    let loopback_port = loopback_server.bound_port().unwrap();
    let body = r#"{"web":{"enabled":true,"host":"0.0.0.0","port":8765,"open_browser":false}}"#;
    let bad_host = put_config(loopback_port, body, "evil.invalid", "");
    let remote_origin = put_config(loopback_port, body, "127.0.0.1", "http://evil.invalid");
    let same_origin = put_config(
        loopback_port,
        body,
        "127.0.0.1",
        &format!("http://127.0.0.1:{loopback_port}"),
    );
    let remote = api_runtime(&config_path, 0, "0.0.0.0");
    let remote_server = start_with_state(&remote, api_store(&remote)).await;
    let remote_response = put_config(remote_server.bound_port().unwrap(), body, "0.0.0.0", "");

    assert!(bad_host.starts_with("HTTP/1.1 403 Forbidden"), "{bad_host}");
    assert!(
        remote_origin.starts_with("HTTP/1.1 403 Forbidden"),
        "{remote_origin}"
    );
    assert!(same_origin.starts_with("HTTP/1.1 200 OK"), "{same_origin}");
    assert!(
        remote_response.starts_with("HTTP/1.1 200 OK"),
        "{remote_response}"
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

struct TestMonitorConfigApplier {
    monitor: Arc<Mutex<MonitorRuntime>>,
}

#[async_trait]
impl RuntimeConfigApplier for TestMonitorConfigApplier {
    async fn apply_runtime_config(&self, config: &RuntimeConfig) {
        self.monitor
            .lock()
            .await
            .apply_runtime_config(config, Utc::now());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_config_update_applies_live_monitor_settings_and_flags_restart_only_changes() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_dir = temp_dir.path().join("journals");
    write_journal_fixture(&journal_dir);
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api live dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, &journal_dir);
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let mut monitor = MonitorRuntime::start(
        &runtime,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::from_runtime_config(&runtime),
        WebStartupStatus::from_current_runtime_config(&runtime),
    )
    .unwrap();
    monitor.process_preload(Utc::now());
    let events = monitor.event_store();
    let monitor = Arc::new(Mutex::new(monitor));
    let applier = Arc::new(TestMonitorConfigApplier {
        monitor: Arc::clone(&monitor),
    });
    let server = start_with_applier(
        &runtime,
        events,
        Some(applier as Arc<dyn RuntimeConfigApplier>),
    )
    .await;
    let port = server.bound_port().unwrap();

    let live_update = r#"{"monitor":{"warn_kill_rate":37,"warn_no_kills_minutes":14,"poll_interval_ms":350},"log_levels":{"scan_easy":0}}"#;
    let live_response = put_config(port, live_update, "127.0.0.1", "http://127.0.0.1:3000");
    let live_json = json_body(&live_response);
    assert!(
        live_response.starts_with("HTTP/1.1 200 OK"),
        "{live_response}"
    );
    assert_eq!(live_json["restart_required"], Value::Bool(false));
    assert_eq!(
        monitor.lock().await.poll_interval(),
        std::time::Duration::from_millis(350)
    );

    let snapshot_response = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    let snapshot_json = json_body(&snapshot_response);
    assert_eq!(
        snapshot_json["thresholds"]["warn_kill_rate_per_hour"],
        Value::from(37)
    );
    assert_eq!(
        snapshot_json["thresholds"]["warn_no_kills_minutes"],
        Value::from(14)
    );

    let restart_update =
        r#"{"web":{"enabled":true,"host":"127.0.0.1","port":9876,"open_browser":false}}"#;
    let restart_response = put_config(port, restart_update, "127.0.0.1", "http://127.0.0.1:3000");
    let restart_json = json_body(&restart_response);
    assert!(
        restart_response.starts_with("HTTP/1.1 200 OK"),
        "{restart_response}"
    );
    assert_eq!(restart_json["restart_required"], Value::Bool(true));
    assert_eq!(
        restart_json["restart_required_sections"],
        serde_json::json!(["Web UI"])
    );

    let reloaded_config_response = request(
        port,
        "GET /api/config HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(
        reloaded_config_response.starts_with("HTTP/1.1 200 OK"),
        "{reloaded_config_response}"
    );
    let reloaded_config_json = json_body(&reloaded_config_response);
    assert_eq!(reloaded_config_json["restart_required"], Value::Bool(true));
    assert_eq!(
        reloaded_config_json["restart_required_sections"],
        serde_json::json!(["Web UI"])
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_config_update_rejects_out_of_range_values_with_422_and_preserves_config() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    let original_toml = "[monitor]\npoll_interval_ms = 1000\n";
    std::fs::write(&config_path, original_toml).unwrap();
    let runtime = api_runtime_for_source(
        ConfigSource::Explicit(ConfigPath::editable(config_path.clone())),
        0,
        "127.0.0.1",
    );
    let server = start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    for (invalid_body, expected_message) in [
        (
            r#"{"monitor":{"poll_interval_ms":0}}"#,
            "monitor.poll_interval_ms must be between 100 and 60000",
        ),
        (
            r#"{"monitor":{"warn_cooldown_minutes":0}}"#,
            "monitor.warn_cooldown_minutes must be between 1 and 1440",
        ),
        (
            r#"{"monitor":{"warn_no_kills_minutes":1441}}"#,
            "monitor.warn_no_kills_minutes must be between 0 and 1440",
        ),
        (
            r#"{"monitor":{"warn_no_kills_initial_minutes":1441}}"#,
            "monitor.warn_no_kills_initial_minutes must be between 0 and 1440",
        ),
        (
            r#"{"monitor":{"warn_kill_rate_delay_minutes":1441}}"#,
            "monitor.warn_kill_rate_delay_minutes must be between 0 and 1440",
        ),
        (
            r#"{"journal":{"recent_files":0}}"#,
            "journal.recent_files must be between 1 and 100",
        ),
        (
            r#"{"web":{"host":"   ","port":8765}}"#,
            "web.host must not be empty",
        ),
        (
            r#"{"matrix":{"enabled":false,"status_update_interval_seconds":5,"clear_access_token":false}}"#,
            "matrix.status_update_interval_seconds must be between 10 and 86400",
        ),
    ] {
        let response = put_config(port, invalid_body, "127.0.0.1", "http://127.0.0.1:3000");
        let response_json = json_body(&response);
        assert!(
            response.starts_with("HTTP/1.1 422 Unprocessable Entity"),
            "body={invalid_body} response={response}"
        );
        assert_eq!(response_json["error"]["code"], "invalid_config_update");
        assert_eq!(response_json["error"]["message"], expected_message);
        assert_eq!(
            std::fs::read_to_string(&config_path).unwrap(),
            original_toml
        );
    }

    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_disabling_matrix_preserves_fields_and_token_across_subsequent_save() {
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

    let disable_matrix = r#"{"matrix":{"enabled":false,"homeserver":"https://matrix.invalid","room_id":"!room:matrix.invalid","mention_user_id":"@cmdr:matrix.invalid","status_update_interval_seconds":90,"access_token_replacement":null,"clear_access_token":false}}"#;
    let disable_response = put_config(port, disable_matrix, "127.0.0.1", "http://127.0.0.1:3000");
    assert!(
        disable_response.starts_with("HTTP/1.1 200 OK"),
        "{disable_response}"
    );
    let disable_json = json_body(&disable_response);
    assert_eq!(
        disable_json["config"]["matrix"]["enabled"],
        Value::Bool(false)
    );
    assert_eq!(
        disable_json["config"]["matrix"]["access_token_present"],
        Value::Bool(true)
    );

    let reloaded_view_response = request(
        port,
        "GET /api/config HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(
        reloaded_view_response.starts_with("HTTP/1.1 200 OK"),
        "{reloaded_view_response}"
    );
    let reloaded_matrix = &json_body(&reloaded_view_response)["config"]["matrix"];
    assert_eq!(reloaded_matrix["enabled"], Value::Bool(false));
    assert_eq!(reloaded_matrix["homeserver"], "https://matrix.invalid");
    assert_eq!(reloaded_matrix["room_id"], "!room:matrix.invalid");
    assert_eq!(reloaded_matrix["mention_user_id"], "@cmdr:matrix.invalid");
    assert_eq!(reloaded_matrix["status_update_interval_seconds"], 90);
    assert_eq!(reloaded_matrix["access_token_present"], Value::Bool(true));

    // Save another setting with the reloaded disabled matrix fields, as the UI form does.
    let save_other_body = serde_json::json!({
        "monitor": { "warn_kill_rate": 25 },
        "matrix": {
            "enabled": reloaded_matrix["enabled"],
            "homeserver": reloaded_matrix["homeserver"],
            "room_id": reloaded_matrix["room_id"],
            "mention_user_id": reloaded_matrix["mention_user_id"],
            "status_update_interval_seconds": reloaded_matrix["status_update_interval_seconds"],
            "access_token_replacement": null,
            "clear_access_token": false
        }
    })
    .to_string();
    let other_response = put_config(port, &save_other_body, "127.0.0.1", "http://127.0.0.1:3000");
    assert!(
        other_response.starts_with("HTTP/1.1 200 OK"),
        "{other_response}"
    );

    let loaded = AppConfig::load_from_path(&config_path).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.config.monitor.warn_kill_rate, 25);
    let matrix = loaded
        .config
        .matrix
        .expect("disabled matrix config should survive reload");
    assert!(!matrix.enabled);
    assert_eq!(matrix.homeserver.as_deref(), Some("https://matrix.invalid"));
    assert_eq!(matrix.room_id.as_deref(), Some("!room:matrix.invalid"));
    assert_eq!(
        matrix.mention_user_id.as_deref(),
        Some("@cmdr:matrix.invalid")
    );
    assert_eq!(matrix.status_update_interval_seconds, 90);
    assert_eq!(matrix.access_token.as_deref(), Some("fixture-access-token"));
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}
