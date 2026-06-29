use ed_sentry::app::EditableConfigUpdate;
use ed_sentry::config::{AppConfig, ConfigPath, ConfigSource, ConfigWriteError};
use serde_json::Value;

use crate::support::{api_runtime, api_store, env_lock, json_body, put_config, write_dist};

#[test]
fn tunnel_config_api_write_keeps_replaces_and_clears_password() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = temp_dir.path().join("config.toml");
    write_tunnel_config(&config, temp_dir.path());

    let keep_update = EditableConfigUpdate {
        tunnel: Some(ed_sentry::app::TunnelConfigEdit {
            provider: Some("cloudflare_quick".to_string()),
            auto_start: Some(true),
            config_password_replacement: None,
            clear_config_password: false,
        }),
        ..EditableConfigUpdate::default()
    };
    AppConfig::write_update_to_source(
        &ConfigSource::Explicit(ConfigPath::editable(config.clone())),
        &keep_update,
    )
    .unwrap();
    let kept = std::fs::read_to_string(&config).unwrap();
    assert!(
        kept.contains("config_password = \"fixture-tunnel-password\""),
        "{kept}"
    );
    assert!(kept.contains("unknown_root"), "{kept}");
    assert!(kept.contains("unknown_tunnel"), "{kept}");
    let preserved = AppConfig::load_from_path(&config).unwrap().config;
    assert_eq!(preserved.web.port, 8765);
    assert_eq!(
        preserved.matrix.unwrap().access_token.as_deref(),
        Some("fixture-access-token")
    );

    let replace_update = EditableConfigUpdate {
        tunnel: Some(ed_sentry::app::TunnelConfigEdit {
            provider: None,
            auto_start: None,
            config_password_replacement: Some("replacement-tunnel-password".to_string()),
            clear_config_password: false,
        }),
        ..EditableConfigUpdate::default()
    };
    AppConfig::write_update_to_source(
        &ConfigSource::Explicit(ConfigPath::editable(config.clone())),
        &replace_update,
    )
    .unwrap();
    let replaced_toml = std::fs::read_to_string(&config).unwrap();
    let redacted_toml = replaced_toml.replace("replacement-tunnel-password", "<redacted>");
    let tunnel_section = redacted_toml.split_once("[tunnel]").unwrap().1.trim();
    println!("manual_qa_tunnel_toml=[tunnel]\n{tunnel_section}");
    assert_eq!(
        AppConfig::load_from_path(&config)
            .unwrap()
            .config
            .tunnel
            .config_password,
        "replacement-tunnel-password"
    );

    let clear_update = EditableConfigUpdate {
        tunnel: Some(ed_sentry::app::TunnelConfigEdit {
            provider: None,
            auto_start: None,
            config_password_replacement: None,
            clear_config_password: true,
        }),
        ..EditableConfigUpdate::default()
    };
    AppConfig::write_update_to_source(
        &ConfigSource::Explicit(ConfigPath::editable(config.clone())),
        &clear_update,
    )
    .unwrap();
    let cleared = std::fs::read_to_string(&config).unwrap();
    assert!(!cleared.contains("config_password"), "{cleared}");
    assert_eq!(
        AppConfig::load_from_path(&config)
            .unwrap()
            .config
            .tunnel
            .config_password,
        ""
    );
    println!("manual_qa_cleanup=tempdir_drop_pending");
}

#[test]
fn tunnel_config_api_write_rejects_conflicting_password_actions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = temp_dir.path().join("config.toml");
    let update = EditableConfigUpdate {
        tunnel: Some(ed_sentry::app::TunnelConfigEdit {
            provider: None,
            auto_start: None,
            config_password_replacement: Some("replacement-tunnel-password".to_string()),
            clear_config_password: true,
        }),
        ..EditableConfigUpdate::default()
    };

    let error = AppConfig::write_update_to_source(
        &ConfigSource::Defaults {
            first_save_target: ConfigPath::first_save(config),
        },
        &update,
    )
    .unwrap_err();

    assert!(matches!(error, ConfigWriteError::InvalidUpdate { .. }));
    assert!(!error.to_string().contains("replacement-tunnel-password"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn tunnel_config_api_view_and_update_do_not_echo_password() {
    let _env = env_lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    write_dist(dist.path(), "api dist");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", dist.path());
    let config_path = temp_dir.path().join("config.toml");
    write_tunnel_config(&config_path, temp_dir.path());
    let runtime = api_runtime(&config_path, 0, "127.0.0.1");
    let server = ed_sentry::web::start_with_state(&runtime, api_store(&runtime)).await;
    let port = server.bound_port().unwrap();
    let replace = r#"{"tunnel":{"provider":"cloudflare_quick","auto_start":true,"config_password_replacement":"replacement-tunnel-password","clear_config_password":false}}"#;

    let response = put_config(port, replace, "127.0.0.1", "http://127.0.0.1:3000");
    let body = json_body(&response);

    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert_eq!(
        body["config"]["tunnel"]["config_password_present"],
        Value::Bool(true)
    );
    assert!(!response.contains("fixture-tunnel-password"), "{response}");
    assert!(
        !response.contains("replacement-tunnel-password"),
        "{response}"
    );
    assert_eq!(
        AppConfig::load_from_path(&config_path)
            .unwrap()
            .config
            .tunnel
            .config_password,
        "replacement-tunnel-password"
    );
    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
}

fn write_tunnel_config(path: &std::path::Path, journal_dir: &std::path::Path) {
    std::fs::write(
        path,
        format!(
            r#"
            unknown_root = "keep"

            [journal]
            folder = "{}"

            [web]
            enabled = true
            host = "127.0.0.1"
            port = 8765
            open_browser = false

            [matrix]
            enabled = true
            homeserver = "https://matrix.invalid"
            room_id = "!room:matrix.invalid"
            access_token = "fixture-access-token"
            status_update_interval_seconds = 60

            [tunnel]
            provider = "cloudflare_quick"
            auto_start = false
            config_password = "fixture-tunnel-password"
            unknown_tunnel = "keep"
            "#,
            journal_dir.display()
        ),
    )
    .unwrap();
}
