use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use tokio::sync::{Mutex, MutexGuard};

use crate::config::WebConfig;

use super::{resolve_assets_for_executable, start};

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

async fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().await
}

fn fixture_index(path: &Path, marker: &str) {
    std::fs::create_dir_all(path).unwrap();
    std::fs::write(
        path.join("index.html"),
        format!("<!doctype html><title>ed-sentry</title><main>{marker}</main>"),
    )
    .unwrap();
}

fn get_root(port: u16) -> String {
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
                if bytes
                    .windows(b"env dist root".len())
                    .any(|w| w == b"env dist root")
                {
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_serves_temp_env_dist_root() {
    let _env = env_lock().await;
    let dist = tempfile::tempdir().unwrap();
    fixture_index(dist.path(), "env dist root");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config = WebConfig {
        enabled: true,
        port: 0,
        ..WebConfig::default()
    };

    let server = start(&config).await;
    let response = get_root(server.bound_port().unwrap());

    assert_eq!(server.status().status_label, "Running");
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains("env dist root"), "{response}");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_resolves_packaged_sibling_webui_before_repo_dist() {
    let _env = env_lock().await;
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
    let package = tempfile::tempdir().unwrap();
    let exe = package.path().join("bin").join("ed-sentry");
    let webui = package.path().join("bin").join("webui");
    std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
    std::fs::write(&exe, "").unwrap();
    fixture_index(&webui, "packaged sibling root");

    let resolved = resolve_assets_for_executable(&exe).unwrap();

    assert_eq!(resolved, webui);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn web_occupied_port_returns_warning_without_server() {
    let _env = env_lock().await;
    let dist = tempfile::tempdir().unwrap();
    fixture_index(dist.path(), "occupied");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let occupied = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = occupied.local_addr().unwrap().port();
    let config = WebConfig {
        enabled: true,
        port,
        ..WebConfig::default()
    };

    let server = start(&config).await;

    assert_eq!(server.status().status_label, "Warning");
    assert!(server.status().message.as_deref().unwrap().contains("bind"));
    assert!(server.bound_port().is_none());
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}
