use ed_sentry::config::WebConfig;
use ed_sentry::web::{resolve_assets_for_executable, start};

use crate::support::{env_lock, get_root, write_dist};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_serves_env_dist_root() {
    let _env = env_lock().await;
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "integration env dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config = WebConfig {
        enabled: true,
        port: 0,
        ..WebConfig::default()
    };

    let server = start(&config).await;
    let response = get_root(server.bound_port().unwrap());

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert!(response.contains("integration env dist"), "{response}");
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn webui_resolves_packaged_sibling_webui() {
    let _env = env_lock().await;
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
    let package = tempfile::tempdir().unwrap();
    let exe = package.path().join("ed-sentry");
    let webui = package.path().join("webui");
    std::fs::write(&exe, "").unwrap();
    write_dist(&webui, "integration packaged dist");

    let resolved = resolve_assets_for_executable(&exe).unwrap();

    assert_eq!(resolved, webui);
}
