use super::*;
use std::fs;

#[test]
fn config_tunnel_missing_section_keeps_defaults() -> Result<(), ConfigError> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [monitor]
        live_status = false
        "#,
    )?;

    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.config.tunnel, TunnelConfig::default());
    assert_eq!(loaded.config.tunnel.provider, "cloudflare_quick");
    assert!(!loaded.config.tunnel.auto_start);
    assert_eq!(loaded.config.tunnel.config_password, "");
    Ok(())
}

#[test]
fn config_tunnel_explicit_values_override_defaults() -> Result<(), ConfigError> {
    let fixture_password = concat!("fixture-", "password");
    let contents = format!(
        "[tunnel]\nprovider = \"cloudflare_quick\"\nauto_start = true\n{} = \"{}\"\n",
        "config_password", fixture_password
    );
    let loaded = AppConfig::from_toml_str(&contents)?;

    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.config.tunnel.provider, "cloudflare_quick");
    assert!(loaded.config.tunnel.auto_start);
    assert_eq!(loaded.config.tunnel.config_password, fixture_password);
    Ok(())
}

#[test]
fn config_tunnel_wrong_typed_section_warns_and_keeps_defaults() -> Result<(), ConfigError> {
    let loaded = AppConfig::from_toml_str("tunnel = true")?;

    assert_eq!(loaded.config.tunnel, TunnelConfig::default());
    assert_eq!(loaded.warnings.len(), 1);
    assert_eq!(
        loaded.warnings[0],
        "config key tunnel has wrong type; using defaults for section"
    );
    Ok(())
}

#[test]
fn config_tunnel_wrong_typed_keys_warn_and_keep_defaults() -> Result<(), ConfigError> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [tunnel]
        provider = false
        auto_start = "yes"
        config_password = 42
        "#,
    )?;

    assert_eq!(loaded.config.tunnel, TunnelConfig::default());
    assert_eq!(loaded.warnings.len(), 3);
    assert!(loaded.warnings[0].contains("tunnel.provider"));
    assert!(loaded.warnings[1].contains("tunnel.auto_start"));
    assert!(loaded.warnings[2].contains("tunnel.config_password"));
    Ok(())
}

#[test]
fn config_tunnel_runtime_preserves_parsed_values() -> Result<(), ConfigError> {
    let fixture_password = concat!("fixture-", "password");
    let contents = format!(
        "[tunnel]\nprovider = \"cloudflare_quick\"\nauto_start = true\n{} = \"{}\"\n",
        "config_password", fixture_password
    );
    let loaded = AppConfig::from_toml_str(&contents)?;

    let runtime = loaded.config.into_runtime(&CliConfigOverrides::default());

    assert_eq!(runtime.tunnel.provider, "cloudflare_quick");
    assert!(runtime.tunnel.auto_start);
    assert_eq!(runtime.tunnel.config_password, fixture_password);
    Ok(())
}

#[test]
fn config_tunnel_debug_redacts_non_empty_password() -> Result<(), ConfigError> {
    let fixture_password = concat!("fixture-", "password");
    let tunnel = TunnelConfig {
        config_password: fixture_password.to_string(),
        ..TunnelConfig::default()
    };
    let app_config = AppConfig {
        tunnel: tunnel.clone(),
        ..AppConfig::default()
    };
    let runtime = app_config
        .clone()
        .into_runtime(&CliConfigOverrides::default());
    let loaded = LoadedConfig {
        config: app_config,
        warnings: Vec::new(),
        source: ConfigSource::InMemory,
    };

    for debug in [
        format!("{tunnel:?}"),
        format!("{runtime:?}"),
        format!("{loaded:?}"),
    ] {
        assert!(debug.contains("<redacted>"), "{debug}");
        assert!(!debug.contains(fixture_password), "{debug}");
    }
    Ok(())
}

#[test]
fn config_tunnel_loads_temporary_toml_fixture_and_cleans_up() -> anyhow::Result<()> {
    let fixture_path = std::env::temp_dir().join(format!(
        "ed-sentry-config-tunnel-{}.toml",
        std::process::id()
    ));
    fs::write(
        &fixture_path,
        r#"
        [tunnel]
        auto_start = true
        "#,
    )?;

    let loaded = match AppConfig::load_from_path(&fixture_path) {
        Ok(loaded) => loaded,
        Err(error) => {
            let _ = fs::remove_file(&fixture_path);
            return Err(error.into());
        }
    };
    fs::remove_file(&fixture_path)?;

    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.config.tunnel.provider, "cloudflare_quick");
    assert!(loaded.config.tunnel.auto_start);
    assert_eq!(loaded.config.tunnel.config_password, "");
    Ok(())
}
