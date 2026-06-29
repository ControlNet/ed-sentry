use std::fs;
use std::path::{Path, PathBuf};

use ed_sentry::config::{JournalConfig, RuntimeConfig};
use ed_sentry::journal::{
    discover_journal_files, journal_folder_from_saved_games, parse_journal_filename_timestamp,
    preload_journal_file, preload_journal_file_with_options, recent_journal_file_choices,
    resolve_journal_folder, select_configured_journal_file, select_newest_journal_file,
    stream_journal_file, JournalError, PreloadOptions,
};
use serde_json::Value;

#[test]
fn journal_discovery_newest_by_filename() {
    let temp_dir = tempfile::tempdir().unwrap();
    write_file(temp_dir.path(), "Journal.240101010101.01.log", "{}\n");
    write_file(temp_dir.path(), "Journal.2026-06-09T140000.01.log", "{}\n");
    write_file(temp_dir.path(), "Journal.2025-12-31T235959.01.log", "{}\n");
    write_file(temp_dir.path(), "Status.json", "{}\n");

    let newest = select_newest_journal_file(temp_dir.path()).unwrap();

    assert_eq!(file_name(&newest.path), "Journal.2026-06-09T140000.01.log");
}

#[test]
fn journal_discovery_empty_dir_error() {
    let temp_dir = tempfile::tempdir().unwrap();

    let error = discover_journal_files(temp_dir.path()).unwrap_err();

    assert!(matches!(error, JournalError::NoJournalFiles { .. }));
    assert_eq!(error.exit_code(), 1);
}

#[test]
fn journal_discovery_recent_file_select_is_deterministic() {
    let temp_dir = tempfile::tempdir().unwrap();
    write_file(temp_dir.path(), "Journal.2024-01-01T010101.01.log", "{}\n");
    write_file(temp_dir.path(), "Journal.2024-01-03T010101.01.log", "{}\n");
    write_file(temp_dir.path(), "Journal.240102010101.01.log", "{}\n");
    write_file(temp_dir.path(), "Status.json", "{}\n");

    let choices = recent_journal_file_choices(temp_dir.path(), 3).unwrap();
    let choice_names = choices
        .iter()
        .map(|choice| (choice.number, file_name(&choice.file.path)))
        .collect::<Vec<_>>();

    assert_eq!(
        choice_names,
        [
            (1, "Journal.2024-01-03T010101.01.log"),
            (2, "Journal.2024-01-01T010101.01.log"),
        ]
    );
}

#[test]
fn journal_discovery_preload_returns_events_and_eof_offset_without_dispatch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    let contents = concat!(
        r#"{"timestamp":"2026-06-09T14:00:00Z","event":"Fileheader"}"#,
        "\n",
        r#"{"timestamp":"2026-06-09T14:01:00Z","event":"Bounty"}"#,
        "\n"
    );
    fs::write(&path, contents).unwrap();
    let notifications_dispatched = 0;

    let preload = preload_journal_file(&path, |line| serde_json::from_str::<Value>(line)).unwrap();

    assert_eq!(preload.records.len(), 2);
    assert_eq!(
        preload.records[0].result.as_ref().unwrap()["event"],
        "Fileheader"
    );
    assert_eq!(preload.eof_offset, contents.len() as u64);
    assert_eq!(notifications_dispatched, 0);
}

#[test]
fn journal_discovery_preload_records_parse_failures_and_reset_hook_flag() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(
        &path,
        concat!(
            r#"{"timestamp":"2026-06-09T14:00:00Z","event":"Fileheader"}"#,
            "\n",
            "not-json\n"
        ),
    )
    .unwrap();

    let preload = preload_journal_file_with_options(
        &path,
        PreloadOptions {
            reset_session_after_preload: true,
        },
        |line| serde_json::from_str::<Value>(line),
    )
    .unwrap();

    assert!(preload.records[0].result.is_ok());
    assert!(preload.records[1].result.is_err());
    assert!(preload.reset_session_after_preload);
}

#[test]
fn journal_discovery_streams_records_without_returning_collection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    let contents = concat!(
        r#"{"timestamp":"2026-06-09T14:00:00Z","event":"Fileheader"}"#,
        "\n",
        "not-json\n",
        r#"{"timestamp":"2026-06-09T14:02:00Z","event":"Bounty"}"#,
        "\n"
    );
    fs::write(&path, contents).unwrap();
    let mut visited = Vec::new();

    let result = stream_journal_file(
        &path,
        |line| serde_json::from_str::<Value>(line),
        |record| {
            visited.push((
                record.line_number,
                record.start_offset,
                record.result.is_ok(),
            ));
            Ok::<(), std::convert::Infallible>(())
        },
    )
    .unwrap();

    assert_eq!(result.eof_offset, contents.len() as u64);
    assert_eq!(visited, [(1, 0, true), (2, 58, false), (3, 67, true),]);
}

#[test]
fn journal_discovery_streaming_callback_error_stops_before_next_record() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(&path, "one\ntwo\nthree\n").unwrap();
    let mut visited = Vec::new();

    let error = stream_journal_file(
        &path,
        |line| Ok::<_, std::convert::Infallible>(line.to_string()),
        |record| {
            visited.push(record.result.unwrap());
            if visited.len() == 2 {
                return Err("stop after second line");
            }
            Ok(())
        },
    )
    .unwrap_err();

    assert!(
        error.to_string().contains("stop after second line"),
        "{error}"
    );
    assert_eq!(visited, ["one", "two"]);
}

#[test]
fn journal_discovery_journal_path_uses_saved_games_root() {
    let path = journal_folder_from_saved_games(Path::new(r"D:\Games\Saved Games"));

    assert!(path.ends_with(Path::new(
        r"D:\Games\Saved Games/Frontier Developments/Elite Dangerous"
    )));
}

#[test]
fn journal_discovery_explicit_folder_bypasses_default_path() {
    let folder = PathBuf::from("/tmp/explicit-journals");
    let config = JournalConfig {
        folder: folder.to_string_lossy().into_owned(),
        recent_files: 10,
    };

    assert_eq!(resolve_journal_folder(&config).unwrap(), folder);
}

#[test]
fn journal_discovery_set_file_bypasses_folder_discovery() {
    let set_file = PathBuf::from("tests/fixtures/journal_combat_bounty.log");
    let config = RuntimeConfig {
        journal: JournalConfig::default(),
        monitor: Default::default(),
        log_levels: Default::default(),
        matrix: None,
        tunnel: ed_sentry::config::TunnelConfig::default(),
        web: ed_sentry::config::WebConfig::default(),
        config_source: Default::default(),
        set_file: Some(set_file.clone()),
        file_select: false,
        reset_session: false,
        debug: false,
    };

    assert_eq!(select_configured_journal_file(&config).unwrap(), set_file);
}

#[test]
fn journal_discovery_parses_legacy_and_iso_filename_timestamps() {
    assert!(parse_journal_filename_timestamp("Journal.240101010101.01.log").is_some());
    assert!(parse_journal_filename_timestamp("Journal.2026-06-09T140000.01.log").is_some());
    assert!(parse_journal_filename_timestamp("Journal.invalid.01.log").is_none());
}

fn write_file(folder: &Path, name: &str, contents: &str) {
    fs::write(folder.join(name), contents).unwrap();
}

fn file_name(path: &Path) -> &str {
    path.file_name().unwrap().to_str().unwrap()
}
