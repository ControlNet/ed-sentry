use ed_sentry::app::EditableConfigUpdate;
use ed_sentry::config::{AppConfig, ConfigPath, ConfigSource, ConfigWriteError};

#[test]
fn webui_config_write_targets_and_toml_editing_are_documented() {
    let temp_dir = tempfile::tempdir().unwrap();
    let explicit = temp_dir.path().join("explicit.toml");
    std::fs::write(
        &explicit,
        r#"
        # keep comment
        unknown_root = "keep"

        [journal]
        folder = "old"
        unknown_journal = "keep"
        "#,
    )
    .unwrap();
    let update = EditableConfigUpdate {
        journal: Some(ed_sentry::app::JournalConfigEdit {
            folder: Some("new".to_string()),
            recent_files: Some(12),
        }),
        ..EditableConfigUpdate::default()
    };
    AppConfig::write_update_to_source(
        &ConfigSource::Explicit(ConfigPath::editable(explicit.clone())),
        &update,
    )
    .unwrap();
    let edited = std::fs::read_to_string(&explicit).unwrap();
    assert!(edited.contains("# keep comment"), "{edited}");
    assert!(edited.contains("unknown_root"), "{edited}");
    assert!(edited.contains("unknown_journal"), "{edited}");
    assert!(edited.contains("folder = \"new\""), "{edited}");

    let implicit = temp_dir.path().join("implicit.toml");
    std::fs::write(&implicit, "").unwrap();
    AppConfig::write_update_to_source(
        &ConfigSource::Implicit(ConfigPath::editable(implicit.clone())),
        &update,
    )
    .unwrap();
    assert_eq!(
        AppConfig::load_from_path(&implicit)
            .unwrap()
            .config
            .journal
            .recent_files,
        12
    );

    let first_save = temp_dir.path().join("first-save.toml");
    AppConfig::write_update_to_source(
        &ConfigSource::Defaults {
            first_save_target: ConfigPath::first_save(first_save.clone()),
        },
        &update,
    )
    .unwrap();
    assert!(first_save.exists());

    let tauri = temp_dir.path().join("tauri").join("config.toml");
    AppConfig::write_update_to_source(
        &ConfigSource::Tauri {
            target: ConfigPath::first_save(tauri.clone()),
        },
        &update,
    )
    .unwrap();
    assert!(tauri.exists());

    let malformed = temp_dir.path().join("malformed.toml");
    let malformed_original = "[monitor\n";
    std::fs::write(&malformed, malformed_original).unwrap();
    let error = AppConfig::write_update_to_source(
        &ConfigSource::Explicit(ConfigPath::editable(malformed.clone())),
        &update,
    )
    .unwrap_err();
    assert!(matches!(error, ConfigWriteError::MalformedToml { .. }));
    assert_eq!(
        std::fs::read_to_string(&malformed).unwrap(),
        malformed_original
    );
}

#[test]
fn webui_config_write_allows_non_loopback_web_host() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = temp_dir.path().join("config.toml");
    let update = EditableConfigUpdate {
        web: Some(ed_sentry::app::WebConfigEdit {
            enabled: Some(true),
            host: Some("0.0.0.0".to_string()),
            port: Some(8765),
            open_browser: Some(false),
        }),
        ..EditableConfigUpdate::default()
    };

    AppConfig::write_update_to_source(
        &ConfigSource::Defaults {
            first_save_target: ConfigPath::first_save(config.clone()),
        },
        &update,
    )
    .unwrap();

    assert_eq!(
        AppConfig::load_from_path(&config).unwrap().config.web.host,
        "0.0.0.0"
    );
}

#[cfg(unix)]
#[test]
fn webui_config_write_permission_failure_is_reported() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let locked = temp_dir.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o500)).unwrap();
    let target = locked.join("config.toml");
    let update = EditableConfigUpdate {
        web: Some(ed_sentry::app::WebConfigEdit {
            enabled: Some(true),
            host: Some("127.0.0.1".to_string()),
            port: Some(8765),
            open_browser: Some(false),
        }),
        ..EditableConfigUpdate::default()
    };

    let error = AppConfig::write_update_to_source(
        &ConfigSource::Defaults {
            first_save_target: ConfigPath::first_save(target),
        },
        &update,
    )
    .unwrap_err();

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(error, ConfigWriteError::Io { .. }));
}
