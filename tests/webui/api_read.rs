use ed_sentry::web::start_with_state;
use serde_json::Value;

use crate::support::{
    api_runtime, api_store, env_lock, json_body, request, runtime_backed_store, write_api_config,
    write_dist, write_journal_fixture,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_snapshot_exposes_configured_journal_folder_path() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let journal_dir = temp_dir.path().join("private-journal-root").join("journal");
    let selected_filename = write_journal_fixture(&journal_dir);
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, &journal_dir);
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, runtime_backed_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    let response = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );

    let raw_journal_dir = journal_dir.to_string_lossy();
    let raw_temp_root = temp_dir.path().to_string_lossy();
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains(raw_journal_dir.as_ref()), "{response}");
    assert!(response.contains(raw_temp_root.as_ref()), "{response}");
    let snapshot_json = json_body(&response);
    assert_eq!(
        snapshot_json["journal_source"]["folder"],
        Value::String(raw_journal_dir.to_string())
    );
    assert_eq!(
        snapshot_json["journal_source"]["status_label"],
        Value::String("Running".to_string())
    );
    assert_eq!(
        snapshot_json["journal_source"]["selected_file"],
        Value::String(selected_filename)
    );
    assert!(snapshot_json.get("session").is_some(), "{snapshot_json}");
    assert!(snapshot_json.get("missions").is_some(), "{snapshot_json}");
    assert!(snapshot_json.get("events").is_some(), "{snapshot_json}");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_snapshot_exposes_journal_folder_for_loopback_request_on_wildcard_bind() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let journal_dir = temp_dir.path().join("private-journal-root").join("journal");
    let selected_filename = write_journal_fixture(&journal_dir);
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, &journal_dir);
    let runtime = api_runtime(&config_path, 0, "0.0.0.0");
    let server = start_with_state(&runtime, runtime_backed_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    let response = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );

    let raw_journal_dir = journal_dir.to_string_lossy();
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains(raw_journal_dir.as_ref()), "{response}");
    let snapshot_json = json_body(&response);
    assert_eq!(
        snapshot_json["journal_source"]["folder"],
        Value::String(raw_journal_dir.to_string())
    );
    assert_eq!(
        snapshot_json["journal_source"]["selected_file"],
        Value::String(selected_filename)
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_snapshot_redacts_journal_folder_for_remote_bind_host() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let journal_dir = temp_dir.path().join("private-journal-root").join("journal");
    let selected_filename = write_journal_fixture(&journal_dir);
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, &journal_dir);
    let runtime = api_runtime(&config_path, 0, "0.0.0.0");
    let server = start_with_state(&runtime, runtime_backed_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    let response = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: 192.168.50.10\r\nConnection: close\r\n\r\n",
    );

    let raw_journal_dir = journal_dir.to_string_lossy();
    let raw_temp_root = temp_dir.path().to_string_lossy();
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(!response.contains(raw_journal_dir.as_ref()), "{response}");
    assert!(!response.contains(raw_temp_root.as_ref()), "{response}");
    let snapshot_json = json_body(&response);
    assert_eq!(
        snapshot_json["journal_source"]["folder"],
        Value::String("Configured journal folder".to_string())
    );
    assert_eq!(
        snapshot_json["journal_source"]["selected_file"],
        Value::String(selected_filename)
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_api_snapshot_config_status_and_config_redaction_work_over_http() {
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

    let snapshot = request(
        port,
        "GET /api/snapshot HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    let config = request(
        port,
        "GET /api/config HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    let web = request(
        port,
        "GET /api/web/status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    let matrix = request(
        port,
        "GET /api/matrix/status HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    let health = request(
        port,
        "GET /api/health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );

    assert!(snapshot.starts_with("HTTP/1.1 200 OK"), "{snapshot}");
    let snapshot_json = json_body(&snapshot);
    assert!(snapshot_json.get("session").is_some(), "{snapshot_json}");
    assert!(snapshot_json.get("missions").is_some(), "{snapshot_json}");
    assert!(snapshot_json.get("events").is_some(), "{snapshot_json}");
    assert!(config.starts_with("HTTP/1.1 200 OK"), "{config}");
    assert!(!config.contains("fixture-access-token"), "{config}");
    assert_eq!(
        json_body(&config)["config"]["matrix"]["access_token_present"],
        Value::Bool(true)
    );
    assert!(web.starts_with("HTTP/1.1 200 OK"), "{web}");
    assert!(matrix.starts_with("HTTP/1.1 200 OK"), "{matrix}");
    assert!(health.starts_with("HTTP/1.1 200 OK"), "{health}");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}
