use std::path::PathBuf;

use super::*;

#[test]
fn cli_config_defaults_match_locked_contract() {
    let config = AppConfig::default();

    assert_eq!(config.journal.folder, "");
    assert_eq!(config.journal.recent_files, 10);
    assert!(!config.monitor.use_utc);
    assert!(config.monitor.live_status);
    assert!(config.monitor.dynamic_title);
    assert_eq!(config.monitor.warn_kill_rate, 20);
    assert_eq!(config.monitor.warn_kill_rate_delay_minutes, 5);
    assert_eq!(config.monitor.warn_no_kills_minutes, 20);
    assert_eq!(config.monitor.warn_no_kills_initial_minutes, 5);
    assert_eq!(config.monitor.warn_cooldown_minutes, 30);
    assert_eq!(config.monitor.duplicate_max, 5);
    assert!(!config.monitor.pirate_names);
    assert!(!config.monitor.bounty_faction);
    assert!(!config.monitor.bounty_value);
    assert!(!config.monitor.extended_stats);
    assert_eq!(config.monitor.min_scan_level, 1);
    assert_eq!(config.monitor.poll_interval_ms, 1000);
    assert_eq!(
        default_level_two_log_fields(&config.log_levels),
        vec![
            "fighter_down",
            "died",
            "cargo_lost",
            "fuel_low",
            "fuel_critical",
            "missions_all",
            "rank_promotion",
            "no_kills",
        ]
    );
    assert_eq!(config.log_levels.summary_faction, 0);
    assert_eq!(config.log_levels.summary_scans, 0);
    assert_eq!(config.log_levels.merits, 0);
    assert_eq!(config.log_levels.duplicate_suppression, 1);
    assert_eq!(config.matrix, None);
    assert_eq!(config.web, WebConfig::default());
    assert!(!config.web.enabled);
    assert_eq!(config.web.host, "127.0.0.1");
    assert_eq!(config.web.port, 8765);
    assert!(!config.web.open_browser);
}

fn default_level_two_log_fields(log_levels: &LogLevelConfig) -> Vec<&'static str> {
    [
        ("scan_incoming", log_levels.scan_incoming),
        ("scan_easy", log_levels.scan_easy),
        ("scan_hard", log_levels.scan_hard),
        ("kill_easy", log_levels.kill_easy),
        ("kill_hard", log_levels.kill_hard),
        ("fighter_hull", log_levels.fighter_hull),
        ("fighter_down", log_levels.fighter_down),
        ("ship_shields", log_levels.ship_shields),
        ("ship_hull", log_levels.ship_hull),
        ("died", log_levels.died),
        ("cargo_lost", log_levels.cargo_lost),
        ("bait_value_low", log_levels.bait_value_low),
        ("security_scan", log_levels.security_scan),
        ("security_attack", log_levels.security_attack),
        ("fuel_report", log_levels.fuel_report),
        ("fuel_low", log_levels.fuel_low),
        ("fuel_critical", log_levels.fuel_critical),
        ("missions", log_levels.missions),
        ("missions_all", log_levels.missions_all),
        ("merits", log_levels.merits),
        ("rank_promotion", log_levels.rank_promotion),
        ("no_kills", log_levels.no_kills),
        ("kill_rate", log_levels.kill_rate),
        ("summary_kills", log_levels.summary_kills),
        ("summary_faction", log_levels.summary_faction),
        ("summary_scans", log_levels.summary_scans),
        ("summary_bounties", log_levels.summary_bounties),
        ("summary_merits", log_levels.summary_merits),
        ("duplicate_suppression", log_levels.duplicate_suppression),
    ]
    .into_iter()
    .filter_map(|(name, level)| (level == 2).then_some(name))
    .collect()
}

#[test]
fn cli_config_toml_values_override_defaults() -> Result<(), ConfigError> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [journal]
        folder = "/journals"

        [monitor]
        live_status = false
        poll_interval_ms = 2500

        [log_levels]
        duplicate_suppression = 2
        "#,
    )?;

    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.config.journal.folder, "/journals");
    assert_eq!(loaded.config.journal.recent_files, 10);
    assert!(!loaded.config.monitor.live_status);
    assert_eq!(loaded.config.monitor.poll_interval_ms, 2500);
    assert_eq!(loaded.config.log_levels.duplicate_suppression, 2);
    Ok(())
}

#[test]
fn cli_config_wrong_typed_keys_warn_and_keep_defaults() -> Result<(), ConfigError> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [monitor]
        poll_interval_ms = "fast"

        [log_levels]
        duplicate_suppression = "loud"
        "#,
    )?;

    assert_eq!(loaded.config.monitor.poll_interval_ms, 1000);
    assert_eq!(loaded.config.log_levels.duplicate_suppression, 1);
    assert_eq!(loaded.warnings.len(), 2);
    assert!(loaded.warnings[0].contains("monitor.poll_interval_ms"));
    assert!(loaded.warnings[1].contains("log_levels.duplicate_suppression"));
    Ok(())
}

#[test]
fn config_web_characterizes_wrong_typed_keys_warn_and_keep_defaults() {
    web::test_support::assert_wrong_typed_keys_warn_and_keep_defaults();
}

#[test]
fn cli_config_cli_overrides_toml_config() -> Result<(), ConfigError> {
    let loaded = AppConfig::from_toml_str(
        r#"
        [journal]
        folder = "/toml/journals"

        [monitor]
        live_status = true
        poll_interval_ms = 1500
        "#,
    )?;

    let runtime = loaded.config.into_runtime(&CliConfigOverrides {
        journal_folder: Some(PathBuf::from("/cli/journals")),
        set_file: Some(PathBuf::from("Journal.log")),
        no_status_line: true,
        poll_interval_ms: Some(500),
        debug: true,
        ..CliConfigOverrides::default()
    });

    assert_eq!(runtime.journal.folder, "/cli/journals");
    assert_eq!(runtime.monitor.poll_interval_ms, 500);
    assert!(!runtime.monitor.live_status);
    assert_eq!(runtime.matrix, None);
    assert_eq!(runtime.set_file, Some(PathBuf::from("Journal.log")));
    assert!(runtime.debug);
    Ok(())
}

#[test]
fn config_web_defaults_to_disabled_localhost() {
    web::test_support::assert_defaults_to_disabled_localhost();
}

#[test]
fn config_source_tracks_write_target() {
    source::test_support::assert_source_tracks_write_target();
}
