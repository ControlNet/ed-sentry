use std::path::PathBuf;

use clap::{CommandFactory, Parser};

use super::*;
use crate::app::runtime::{status_cadence_from_config, watch_journal_folder_display};
use crate::delivery::StatusCadence;

#[test]
fn cli_config_help_lists_locked_flags() {
    let help = Cli::command().render_help().to_string();

    assert!(help.contains("--journal <folder>"));
    assert!(help.contains("--set-file <file>"));
    assert!(help.contains("--file-select"));
    assert!(help.contains("--reset-session"));
    assert!(help.contains("--debug"));
    assert!(help.contains("--config <file>"));
    assert!(help.contains("--no-status-line"));
    assert!(help.contains("--poll-interval-ms <ms>"));
    assert!(help.contains("--replay"));
    assert!(!help.contains("Commands:"));
}

#[test]
fn cli_config_watch_accepts_poll_interval() {
    let cli = Cli::try_parse_from([
        "ed-sentry",
        "--journal",
        "/journals",
        "--poll-interval-ms",
        "250",
    ])
    .unwrap();
    let command = build_runtime_command(cli).unwrap();

    assert_eq!(command.mode, Mode::Watch);
    assert_eq!(command.config.journal.folder, "/journals");
    assert_eq!(command.config.monitor.poll_interval_ms, 250);
}

#[test]
fn cli_config_replay_flag_enables_replay_mode() {
    let cli = Cli::try_parse_from([
        "ed-sentry",
        "--replay",
        "--set-file",
        "tests/fixtures/journal_combat_bounty.log",
        "--no-status-line",
    ])
    .unwrap();
    let command = build_runtime_command(cli).unwrap();

    assert_eq!(command.mode, Mode::Replay);
    assert_eq!(
        command.config.set_file,
        Some(PathBuf::from("tests/fixtures/journal_combat_bounty.log"))
    );
    assert!(!command.config.monitor.live_status);
}

#[test]
fn cli_config_without_replay_defaults_to_watch() {
    let cli = Cli::try_parse_from([
        "ed-sentry",
        "--journal",
        "/journals",
        "--set-file",
        "Journal.log",
    ])
    .unwrap();
    let command = build_runtime_command(cli).unwrap();

    assert_eq!(command.mode, Mode::Watch);
    assert_eq!(command.config.journal.folder, "/journals");
    assert_eq!(command.config.set_file, Some(PathBuf::from("Journal.log")));
}

#[test]
fn cli_config_watch_display_uses_explicit_folder() {
    let config = RuntimeConfig {
        journal: crate::config::JournalConfig {
            folder: "/journals".to_string(),
            recent_files: 10,
        },
        monitor: Default::default(),
        log_levels: Default::default(),
        matrix: None,
        tunnel: crate::config::TunnelConfig::default(),
        web: crate::config::WebConfig::default(),
        config_source: Default::default(),
        set_file: None,
        file_select: false,
        reset_session: false,
        debug: false,
    };

    assert_eq!(watch_journal_folder_display(&config), "/journals");
}

#[test]
fn cli_config_status_cadence_uses_matrix_interval_or_default() {
    let default_config = RuntimeConfig {
        journal: Default::default(),
        monitor: Default::default(),
        log_levels: Default::default(),
        matrix: None,
        tunnel: crate::config::TunnelConfig::default(),
        web: crate::config::WebConfig::default(),
        config_source: Default::default(),
        set_file: None,
        file_select: false,
        reset_session: false,
        debug: false,
    };
    let matrix_config = RuntimeConfig {
        matrix: Some(crate::config::MatrixConfig {
            status_update_interval_seconds: 45,
            ..crate::config::MatrixConfig::default()
        }),
        ..default_config.clone()
    };

    assert_eq!(
        status_cadence_from_config(&default_config),
        StatusCadence::from_interval_seconds(60)
    );
    assert_eq!(
        status_cadence_from_config(&matrix_config),
        StatusCadence::from_interval_seconds(45)
    );
}

#[cfg(not(windows))]
#[test]
fn cli_config_watch_display_explains_unavailable_default_folder() {
    let config = RuntimeConfig {
        journal: Default::default(),
        monitor: Default::default(),
        log_levels: Default::default(),
        matrix: None,
        tunnel: crate::config::TunnelConfig::default(),
        web: crate::config::WebConfig::default(),
        config_source: Default::default(),
        set_file: None,
        file_select: false,
        reset_session: false,
        debug: false,
    };

    assert_eq!(
        watch_journal_folder_display(&config),
        "<Windows Saved Games known folder unavailable>"
    );
}

#[test]
fn cli_config_replay_rejects_poll_interval_ms() {
    let cli = Cli::try_parse_from([
        "ed-sentry",
        "--replay",
        "--poll-interval-ms",
        "1000",
        "--set-file",
        "Journal.log",
    ])
    .unwrap();
    let error = build_runtime_command(cli).unwrap_err();

    assert!(error.message.contains("--poll-interval-ms has no effect"));
}

#[test]
fn cli_config_replay_requires_set_file() {
    let cli = Cli::try_parse_from(["ed-sentry", "--replay"]).unwrap();
    let error = build_runtime_command(cli).unwrap_err();

    assert!(error.message.contains("replay requires --set-file"));
}

#[test]
fn cli_config_replay_rejects_journal_folder() {
    let cli = Cli::try_parse_from([
        "ed-sentry",
        "--replay",
        "--journal",
        "/journals",
        "--set-file",
        "Journal.log",
    ])
    .unwrap();
    let error = build_runtime_command(cli).unwrap_err();

    assert!(error.message.contains("replay does not accept --journal"));
}
