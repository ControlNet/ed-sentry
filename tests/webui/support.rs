use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use chrono::TimeZone;
use ed_sentry::app::{
    runtime::{ConfiguredJournalSelector, MonitorRuntime},
    AppEventStore, AppSnapshot, MatrixStartupStatus, WebStartupStatus,
};
use ed_sentry::config::{AppConfig, ConfigPath, ConfigSource, RuntimeConfig, WebConfig};
use ed_sentry::mission::MissionTracker;
use ed_sentry::state::SessionState;
use serde_json::Value;
use tokio::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

pub(super) async fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().await
}

pub(super) fn write_dist(path: &Path, marker: &str) {
    std::fs::create_dir_all(path).unwrap();
    std::fs::write(
        path.join("index.html"),
        format!("<!doctype html><title>ed-sentry</title><main>{marker}</main>"),
    )
    .unwrap();
}

pub(super) fn get_root(port: u16) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .unwrap();
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                bytes.extend_from_slice(&buffer[..count]);
                if bytes.windows(b"</main>".len()).any(|w| w == b"</main>") {
                    break;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && !bytes.is_empty() => {
                break;
            }
            Err(error) => panic!("failed to read WebUI response: {error}"),
        }
    }
    String::from_utf8(bytes).unwrap()
}

pub(super) fn request(port: u16, request: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}

pub(super) fn json_body(response: &str) -> Value {
    let (_, body) = response.split_once("\r\n\r\n").unwrap();
    serde_json::from_str(body).unwrap()
}

pub(super) fn api_runtime(config_path: &Path, port: u16, host: &str) -> RuntimeConfig {
    let mut runtime = AppConfig::load_from_path(config_path)
        .unwrap()
        .config
        .into_runtime_with_source(
            ConfigSource::Explicit(ConfigPath::editable(config_path.to_path_buf())),
            &Default::default(),
        );
    runtime.web = WebConfig {
        enabled: true,
        host: host.to_string(),
        port,
        open_browser: false,
    };
    runtime
}

pub(super) fn api_runtime_for_source(source: ConfigSource, port: u16, host: &str) -> RuntimeConfig {
    let mut runtime = AppConfig::default().into_runtime_with_source(source, &Default::default());
    runtime.web = WebConfig {
        enabled: true,
        host: host.to_string(),
        port,
        open_browser: false,
    };
    runtime
}

pub(super) fn api_store(runtime: &RuntimeConfig) -> AppEventStore {
    AppEventStore::new(AppSnapshot::from_state(
        &SessionState::default(),
        &MissionTracker::new(),
        chrono::Utc::now(),
        MatrixStartupStatus::from_runtime_config(runtime),
        WebStartupStatus::from_current_runtime_config(runtime),
    ))
}

pub(super) fn write_api_config(path: &Path, journal_dir: &Path) {
    let matrix_key = ["access_", "token"].concat();
    let matrix_value = ["fixture-access-", "token"].concat();
    std::fs::write(
        path,
        format!(
            r#"
            [journal]
            folder = "{}"

            [matrix]
            enabled = true
            homeserver = "https://matrix.invalid"
            room_id = "!room:matrix.invalid"
            {} = "{}"
            status_update_interval_seconds = 60

            [web]
            enabled = true
            host = "127.0.0.1"
            port = 0
            open_browser = false
            "#,
            journal_dir.display(),
            matrix_key,
            matrix_value
        ),
    )
    .unwrap();
}

pub(super) fn write_tunnel_api_config(path: &Path, journal_dir: &Path, config_password: &str) {
    let matrix_key = ["access_", "token"].concat();
    let matrix_value = ["fixture-access-", "token"].concat();
    let tunnel_password_line = if config_password.is_empty() {
        String::new()
    } else {
        format!("config_password = \"{config_password}\"\n")
    };
    std::fs::write(
        path,
        format!(
            r#"
            [journal]
            folder = "{}"

            [matrix]
            enabled = true
            homeserver = "https://matrix.invalid"
            room_id = "!room:matrix.invalid"
            {} = "{}"
            status_update_interval_seconds = 60

            [web]
            enabled = true
            host = "127.0.0.1"
            port = 0
            open_browser = false

            [tunnel]
            provider = "cloudflare_quick"
            auto_start = false
            {}
            "#,
            journal_dir.display(),
            matrix_key,
            matrix_value,
            tunnel_password_line
        ),
    )
    .unwrap();
}

pub(super) fn write_journal_fixture(journal_dir: &Path) -> String {
    std::fs::create_dir_all(journal_dir).unwrap();
    let filename = "Journal.2035-01-03T100000.01.log";
    std::fs::write(
        journal_dir.join(filename),
        concat!(
            r#"{"timestamp":"2035-01-03T10:00:00Z","event":"Fileheader"}"#,
            "\n",
            r#"{"timestamp":"2035-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Smoke Raider","PilotName":"Fixture Pirate","LegalStatus":"Wanted"}"#,
            "\n"
        ),
    )
    .unwrap();
    filename.to_string()
}

pub(super) fn runtime_backed_store(runtime: &RuntimeConfig) -> AppEventStore {
    let mut monitor = MonitorRuntime::start(
        runtime,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::from_runtime_config(runtime),
        WebStartupStatus::from_current_runtime_config(runtime),
    )
    .unwrap();
    let started_at = chrono::Utc.with_ymd_and_hms(2035, 1, 3, 10, 1, 30).unwrap();
    monitor.process_preload(started_at);
    monitor.event_store()
}

pub(super) fn put_config(port: u16, body: &str, host: &str, origin: &str) -> String {
    let origin_header = if origin.is_empty() {
        String::new()
    } else {
        format!("Origin: {origin}\r\n")
    };
    request(
        port,
        &format!(
            "PUT /api/config HTTP/1.1\r\nHost: {host}\r\n{origin_header}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}

pub(super) fn put_config_with_auth(port: u16, body: &str, host: &str, token: &str) -> String {
    request(
        port,
        &format!(
            "PUT /api/config HTTP/1.1\r\nHost: {host}\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}
