#[path = "cli_config/capture_output.rs"]
mod capture_output;
#[path = "cli_config/capture_text.rs"]
mod capture_text;
#[path = "cli_config/command.rs"]
mod command;
#[path = "cli_config/journal.rs"]
mod journal;

use assert_cmd::Command;
use ed_sentry::build_info::APP_BUILD_VERSION;
use predicates::prelude::*;
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;

const WATCH_READINESS_DEADLINE: Duration = Duration::from_secs(10);

fn expected_banner() -> String {
    format!("ed-sentry {APP_BUILD_VERSION} by CMDR ControlNet")
}

#[test]
fn cli_config_replay_flag_and_default_watch_mode() {
    let mut before = Command::cargo_bin("ed-sentry-core").unwrap();
    before
        .args([
            "--replay",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--no-status-line",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_banner()))
        .stdout(predicate::str::contains(
            "Journal file: journal_combat_bounty.log",
        ))
        .stdout(predicate::str::contains(
            "Starting... (Press Ctrl+C to stop)",
        ));

    let temp_dir = tempfile::tempdir().unwrap();
    journal::write_minimal_journal(temp_dir.path());
    let mut after = command::binary_command();
    after
        .arg("--journal")
        .arg(temp_dir.path())
        .arg("--poll-interval-ms")
        .arg("222")
        .arg("--no-status-line");
    let stdout = capture_text::capture_watch_startup(after);
    assert!(stdout.contains(&format!("Journal folder: {}", temp_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

#[test]
fn cli_config_default_watch_mode_binary() {
    let temp_dir = tempfile::tempdir().unwrap();
    journal::write_minimal_journal(temp_dir.path());
    let mut command = command::binary_command();
    command
        .arg("--journal")
        .arg(temp_dir.path())
        .arg("--no-status-line");

    let stdout = capture_text::capture_watch_startup(command);
    assert!(stdout.contains(&expected_banner()));
    assert!(stdout.contains(&format!("Journal folder: {}", temp_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

#[test]
fn cli_config_watch_tails_until_stopped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-03T100000.01.log");
    std::fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-03T10:00:00Z","event":"Fileheader"}"#,
    )
    .unwrap();

    let mut command = command::binary_command();
    command
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line");

    let output = capture_output::capture_watch_output_until(
        command,
        &["Starting... (Press Ctrl+C to stop)"],
        &[],
        WATCH_READINESS_DEADLINE,
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn cli_config_watch_preloads_existing_event_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-03T100000.01.log");
    std::fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-03T10:00:00Z","event":"Fileheader"}"#,
            "\n",
            r#"{"timestamp":"2035-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Viper Mk III","PilotName":"Preload Raider","LegalStatus":"Wanted"}"#,
            "\n",
            r#"{"timestamp":"2035-01-03T10:03:00Z","event":"Bounty","TotalReward":6400,"Target":"viper","Target_Localised":"Viper Mk III","VictimFaction":"Preload Raiders"}"#,
            "\n"
        ),
    )
    .unwrap();

    let mut command = command::binary_command();
    command
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line");
    let output = capture_output::capture_watch_output_until(
        command,
        &["Scan: Viper Mk III", "Kill: Viper Mk III"],
        &[],
        WATCH_READINESS_DEADLINE,
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("Scan: Viper Mk III"));
    assert!(stdout.contains("Kill: Viper Mk III"));
}

#[test]
fn cli_config_file_select_chooses_recent_journal_from_stdin() {
    let temp_dir = tempfile::tempdir().unwrap();
    journal::write_named_journal(
        temp_dir.path(),
        "Journal.2035-01-03T100000.01.log",
        "Old Selected Viper",
    );
    journal::write_named_journal(
        temp_dir.path(),
        "Journal.2035-01-04T100000.01.log",
        "Newest Cobra",
    );

    let mut command = command::binary_command();
    command
        .arg("--journal")
        .arg(temp_dir.path())
        .arg("--file-select")
        .arg("--no-status-line")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(b"2\n").unwrap();
    drop(child.stdin.take());

    let mut watch = capture_output::RunningWatch::from_child(child);
    watch.wait_for_output(
        &["1:", "2:", "Scan: Old Selected Viper"],
        &[],
        WATCH_READINESS_DEADLINE,
    );
    let output = watch.stop();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("1:"), "{stdout}");
    assert!(stdout.contains("2:"), "{stdout}");
    assert!(
        stdout.contains("Journal.2035-01-03T100000.01.log"),
        "{stdout}"
    );
    assert!(stdout.contains("Scan: Old Selected Viper"), "{stdout}");
    assert!(!stdout.contains("Scan: Newest Cobra"), "{stdout}");
}
