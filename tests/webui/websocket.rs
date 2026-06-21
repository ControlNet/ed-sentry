use ed_sentry::web::start_with_state;
use futures_util::StreamExt;
use serde_json::Value;

use crate::support::{
    api_runtime, api_store, env_lock, runtime_backed_store, write_api_config, write_dist,
    write_journal_fixture,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_websocket_hello_redacts_configured_journal_folder_path() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "ws dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let journal_dir = temp_dir.path().join("private-journal-root").join("journal");
    let selected_filename = write_journal_fixture(&journal_dir);
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, &journal_dir);
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = start_with_state(&runtime, runtime_backed_store(&runtime)).await;
    let port = server.bound_port().unwrap();

    let (mut socket, _) =
        tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/api/events"))
            .await
            .unwrap();
    let hello = socket.next().await.unwrap().unwrap().into_text().unwrap();

    let raw_journal_dir = journal_dir.to_string_lossy();
    let raw_temp_root = temp_dir.path().to_string_lossy();
    assert!(!hello.contains(raw_journal_dir.as_ref()), "{hello}");
    assert!(!hello.contains(raw_temp_root.as_ref()), "{hello}");
    let hello_json: Value = serde_json::from_str(&hello).unwrap();
    assert_eq!(hello_json["type"], "hello");
    assert_eq!(hello_json["version"], 1);
    assert_eq!(
        hello_json["snapshot"]["journal_source"]["folder"],
        Value::String("Configured journal folder".to_string())
    );
    assert_eq!(
        hello_json["snapshot"]["journal_source"]["selected_file"],
        Value::String(selected_filename)
    );
    assert!(
        hello_json["snapshot"].get("session").is_some(),
        "{hello_json}"
    );
    assert!(hello_json.get("event_feed").is_some(), "{hello_json}");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_websocket_sends_hello_buffer_and_live_update() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "ws dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_api_config(&config_path, temp_dir.path());
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let store = api_store(&runtime);
    store.record_lifecycle("buffered_event", "Buffered event", chrono::Utc::now());
    let server = start_with_state(&runtime, store.clone()).await;
    let port = server.bound_port().unwrap();

    let (mut socket, _) =
        tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}/api/events"))
            .await
            .unwrap();
    let hello = socket.next().await.unwrap().unwrap().into_text().unwrap();
    store.record_lifecycle("live_event", "Live event", chrono::Utc::now());
    let live = socket.next().await.unwrap().unwrap().into_text().unwrap();

    let hello_json: Value = serde_json::from_str(&hello).unwrap();
    let live_json: Value = serde_json::from_str(&live).unwrap();
    assert_eq!(hello_json["type"], "hello");
    assert_eq!(hello_json["version"], 1);
    assert!(
        hello_json["snapshot"].get("session").is_some(),
        "{hello_json}"
    );
    assert_eq!(hello_json["event_feed"][0]["event_type"], "buffered_event");
    assert_eq!(live_json["type"], "event");
    assert_eq!(live_json["version"], 1);
    assert_eq!(live_json["item"]["event_type"], "live_event");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}
