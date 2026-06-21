use super::*;
use anyhow::{anyhow, Result};

#[test]
fn config_matrix_missing_section_defaults_to_none() -> Result<()> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [monitor]
        live_status = false
        "#,
    )?;

    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.config.matrix, None);
    Ok(())
}

#[test]
fn config_matrix_disabled_is_silent_none() -> Result<()> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [matrix]
        enabled = false
        homeserver = 42
        "#,
    )?;

    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.config.matrix, None);
    Ok(())
}

#[test]
fn config_matrix_enabled_preserves_present_fields_for_runtime_validation() -> Result<()> {
    let loaded = AppConfig::from_toml_str(concat!(
        r#"
        [matrix]
        enabled = true
        homeserver = "https://matrix.invalid"
        user_id = "@fixture:matrix.invalid"
        room_id = "!fixture-room:matrix.invalid"
        access_"#,
        "token",
        r#" = "fixture-value"
        mention_user_id = "@mention-fixture:matrix.invalid"
        status_update_interval_seconds = 45
        "#,
    ))?;

    let matrix = require_some(loaded.config.matrix, "matrix config should be present")?;
    assert!(loaded.warnings.is_empty());
    assert!(matrix.enabled);
    assert_eq!(matrix.homeserver.as_deref(), Some("https://matrix.invalid"));
    assert_eq!(matrix.user_id.as_deref(), Some("@fixture:matrix.invalid"));
    assert_eq!(
        matrix.room_id.as_deref(),
        Some("!fixture-room:matrix.invalid")
    );
    assert_eq!(matrix.access_token.as_deref(), Some("fixture-value"));
    assert_eq!(
        matrix.mention_user_id.as_deref(),
        Some("@mention-fixture:matrix.invalid")
    );
    assert_eq!(matrix.status_update_interval_seconds, 45);
    Ok(())
}

#[test]
fn config_matrix_optional_mention_and_status_interval_default() -> Result<()> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [matrix]
        enabled = true
        homeserver = "https://matrix.invalid"
        "#,
    )?;

    let matrix = require_some(loaded.config.matrix, "matrix config should be present")?;
    assert!(loaded.warnings.is_empty());
    assert_eq!(matrix.homeserver.as_deref(), Some("https://matrix.invalid"));
    assert_eq!(matrix.mention_user_id, None);
    assert_eq!(matrix.status_update_interval_seconds, 60);
    Ok(())
}

#[test]
fn config_matrix_enabled_wrong_typed_keys_warn_and_keep_defaults() -> Result<()> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [matrix]
        enabled = true
        homeserver = false
        user_id = 42
        room_id = []
        access_token = {}
        mention_user_id = 99
        status_update_interval_seconds = "often"
        "#,
    )?;

    let matrix = require_some(loaded.config.matrix, "matrix config should be present")?;
    assert_eq!(matrix.homeserver, None);
    assert_eq!(matrix.user_id, None);
    assert_eq!(matrix.room_id, None);
    assert_eq!(matrix.access_token, None);
    assert_eq!(matrix.mention_user_id, None);
    assert_eq!(matrix.status_update_interval_seconds, 60);
    assert_eq!(loaded.warnings.len(), 6);
    assert!(loaded.warnings[0].contains("matrix.homeserver"));
    assert!(loaded.warnings[1].contains("matrix.user_id"));
    assert!(loaded.warnings[2].contains("matrix.room_id"));
    assert!(loaded.warnings[3].contains("matrix.access_token"));
    assert!(loaded.warnings[4].contains("matrix.mention_user_id"));
    assert!(
        loaded.warnings[5].contains("matrix.status_update_interval_seconds"),
        "{:?}",
        loaded.warnings
    );
    Ok(())
}

#[test]
fn matrix_runtime_config_complete_enabled_config_preserves_runtime_fields() -> Result<()> {
    let loaded = AppConfig::from_toml_str(concat!(
        r#"
        [matrix]
        enabled = true
        homeserver = "https://matrix.invalid"
        user_id = "@fixture:matrix.invalid"
        room_id = "!fixture-room:matrix.invalid"
        access_"#,
        "token",
        r#" = "fixture-value"
        mention_user_id = "@mention-fixture:matrix.invalid"
        status_update_interval_seconds = 45
        "#,
    ))?;

    let runtime = loaded
        .config
        .into_runtime(&CliConfigOverrides::default())
        .matrix_runtime();
    let matrix = require_some(runtime.config, "matrix runtime config should be present")?;
    assert!(runtime.warnings.is_empty());
    assert_eq!(matrix.homeserver, "https://matrix.invalid");
    assert_eq!(matrix.user_id, "@fixture:matrix.invalid");
    assert_eq!(matrix.room_id, "!fixture-room:matrix.invalid");
    assert_eq!(matrix.access_token, "fixture-value");
    assert_eq!(
        matrix.mention_user_id.as_deref(),
        Some("@mention-fixture:matrix.invalid")
    );
    assert_eq!(matrix.status_update_interval_seconds, 45);
    Ok(())
}

#[test]
fn matrix_runtime_config_redacts_access_token() -> Result<()> {
    let matrix = MatrixConfig {
        homeserver: Some("https://matrix.invalid".to_string()),
        user_id: Some("@fixture:matrix.invalid".to_string()),
        room_id: Some("!fixture-room:matrix.invalid".to_string()),
        access_token: Some("fixture-value".to_string()),
        ..MatrixConfig::default()
    };
    let runtime = matrix.to_runtime_config();
    let runtime_config = require_some(
        runtime.config.clone(),
        "matrix runtime config should be present",
    )?;
    let app_config = AppConfig {
        matrix: Some(matrix.clone()),
        ..AppConfig::default()
    };
    let full_runtime = app_config
        .clone()
        .into_runtime(&CliConfigOverrides::default());
    let loaded = LoadedConfig {
        config: app_config,
        warnings: Vec::new(),
        source: ConfigSource::InMemory,
    };

    for debug in [
        format!("{matrix:?}"),
        format!("{runtime_config:?}"),
        format!("{runtime:?}"),
        format!("{full_runtime:?}"),
        format!("{loaded:?}"),
    ] {
        assert!(debug.contains("<redacted>"), "{debug}");
        assert!(!debug.contains("fixture-value"), "{debug}");
    }
    Ok(())
}

#[test]
fn matrix_enabled_missing_required_field_disables_with_warning() -> Result<()> {
    let loaded = AppConfig::from_toml_str(concat!(
        r#"
        [matrix]
        enabled = true
        homeserver = "https://matrix.invalid"
        room_id = "!fixture-room:matrix.invalid"
        access_"#,
        "token",
        r#" = "fixture-value"
        "#,
    ))?;

    let runtime = loaded
        .config
        .into_runtime(&CliConfigOverrides::default())
        .matrix_runtime();

    assert_eq!(runtime.config, None);
    assert_eq!(runtime.warnings.len(), 1);
    assert_eq!(
        runtime.warnings[0],
        "Matrix delivery disabled for this run: missing required matrix config field(s): user_id"
    );
    assert!(!runtime.warnings[0].contains("fixture-value"));
    assert!(!runtime.warnings[0].contains('\n'));
    Ok(())
}

#[test]
fn matrix_runtime_config_uses_fixed_device_id() -> Result<()> {
    let matrix = MatrixConfig {
        homeserver: Some("https://matrix.invalid".to_string()),
        user_id: Some("@fixture:matrix.invalid".to_string()),
        room_id: Some("!fixture-room:matrix.invalid".to_string()),
        access_token: Some("fixture-value".to_string()),
        ..MatrixConfig::default()
    }
    .to_runtime_config()
    .config;

    let matrix = require_some(matrix, "matrix runtime config should be present")?;

    assert_eq!(matrix.device_id(), "EDAFKDASHBOARD");
    Ok(())
}

#[test]
fn matrix_runtime_config_missing_or_disabled_matrix_is_silent() {
    let missing = matrix_runtime_config(&None);
    let disabled = MatrixConfig {
        enabled: false,
        ..MatrixConfig::default()
    }
    .to_runtime_config();

    assert_eq!(missing.config, None);
    assert!(missing.warnings.is_empty());
    assert_eq!(disabled.config, None);
    assert!(disabled.warnings.is_empty());
}

fn require_some<T>(value: Option<T>, message: &'static str) -> Result<T> {
    value.ok_or_else(|| anyhow!(message))
}
