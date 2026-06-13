use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;

#[test]
fn cli_config_help_lists_required_flags() {
    let mut command = Command::cargo_bin("ed-afk-dashboard").unwrap();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--journal <folder>"))
        .stdout(predicate::str::contains("--set-file <file>"))
        .stdout(predicate::str::contains("--file-select"))
        .stdout(predicate::str::contains("--reset-session"))
        .stdout(predicate::str::contains("--debug"))
        .stdout(predicate::str::contains("--config <file>"))
        .stdout(predicate::str::contains("--no-status-line"))
        .stdout(predicate::str::contains("--poll-interval-ms <ms>"))
        .stdout(predicate::str::contains("--replay"));
}

#[test]
fn cli_config_bad_flag_exits_clap_code_2() {
    let mut command = Command::cargo_bin("ed-afk-dashboard").unwrap();

    command
        .arg("--definitely-not-a-flag")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn cli_config_replay_rejects_poll_interval_ms_as_no_effect() {
    let mut command = Command::cargo_bin("ed-afk-dashboard").unwrap();

    command
        .args([
            "--replay",
            "--poll-interval-ms",
            "1000",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "--poll-interval-ms has no effect with --replay",
        ));
}

#[test]
fn cli_config_replay_requires_set_file() {
    let mut command = Command::cargo_bin("ed-afk-dashboard").unwrap();

    command
        .arg("--replay")
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "replay requires --set-file <file>",
        ));
}

#[test]
fn cli_config_replay_rejects_journal() {
    let mut command = Command::cargo_bin("ed-afk-dashboard").unwrap();

    command
        .args([
            "--replay",
            "--journal",
            "/journals",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("replay does not accept --journal"));
}

#[test]
fn cli_config_replay_flag_and_default_watch_mode() {
    let mut before = Command::cargo_bin("ed-afk-dashboard").unwrap();
    before
        .args([
            "--replay",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--no-status-line",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ED AFK Dashboard v260421 by CMDR PSIPAB",
        ))
        .stdout(predicate::str::contains(
            "Journal file: journal_combat_bounty.log",
        ))
        .stdout(predicate::str::contains(
            "Starting... (Press Ctrl+C to stop)",
        ));

    let temp_dir = tempfile::tempdir().unwrap();
    write_minimal_journal(temp_dir.path());
    let mut after = binary_command();
    after
        .arg("--journal")
        .arg(temp_dir.path())
        .arg("--poll-interval-ms")
        .arg("222")
        .arg("--no-status-line");
    let stdout = capture_watch_startup(after);
    assert!(stdout.contains(&format!("Journal folder: {}", temp_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

#[test]
fn cli_config_default_watch_mode_binary() {
    let temp_dir = tempfile::tempdir().unwrap();
    write_minimal_journal(temp_dir.path());
    let mut command = binary_command();
    command
        .arg("--journal")
        .arg(temp_dir.path())
        .arg("--no-status-line");

    let stdout = capture_watch_startup(command);
    assert!(stdout.contains("ED AFK Dashboard v260421 by CMDR PSIPAB"));
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

    let mut command = binary_command();
    command
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();

    std::thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch exited instead of tailing the selected Journal file"
    );
    child.kill().ok();
    child.wait().unwrap();
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

    let mut command = binary_command();
    command
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line");
    let stdout = capture_watch_startup(command);

    assert!(stdout.contains("Scan: Viper Mk III"));
    assert!(stdout.contains("Kill: Viper Mk III"));
}

#[test]
fn cli_config_file_select_chooses_recent_journal_from_stdin() {
    let temp_dir = tempfile::tempdir().unwrap();
    write_named_journal(
        temp_dir.path(),
        "Journal.2035-01-03T100000.01.log",
        "Old Selected Viper",
    );
    write_named_journal(
        temp_dir.path(),
        "Journal.2035-01-04T100000.01.log",
        "Newest Cobra",
    );

    let mut command = binary_command();
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

    std::thread::sleep(Duration::from_millis(150));
    assert!(child.try_wait().unwrap().is_none());
    child.kill().ok();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("1:"), "{stdout}");
    assert!(stdout.contains("2:"), "{stdout}");
    assert!(
        stdout.contains("Journal.2035-01-03T100000.01.log"),
        "{stdout}"
    );
    assert!(stdout.contains("Scan: Old Selected Viper"), "{stdout}");
    assert!(!stdout.contains("Scan: Newest Cobra"), "{stdout}");
}

#[test]
fn cli_config_debug_prints_runtime_diagnostics() {
    let mut command = Command::cargo_bin("ed-afk-dashboard").unwrap();

    command
        .args([
            "--debug",
            "--replay",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--no-status-line",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Debug: replaying Journal file"))
        .stderr(predicate::str::contains("Debug: replay loaded"));
}

#[test]
fn cli_config_startup_sanitizes_untrusted_commander_text() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("journal_sanitized_startup.log");
    std::fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-03T10:00:00Z","event":"Commander","Name":"Cmdr Fixture\nInjected\u001b[2K"}"#,
            "\n"
        ),
    )
    .unwrap();

    let output = Command::cargo_bin("ed-afk-dashboard")
        .unwrap()
        .args([
            "--replay",
            "--set-file",
            journal_path.to_str().unwrap(),
            "--no-status-line",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Commander name: Cmdr Fixture Injected [2K"),
        "{stdout:?}"
    );
    assert!(!stdout.contains("\nInjected"), "{stdout:?}");
    assert!(!stdout.contains('\u{1b}'), "{stdout:?}");
}

#[test]
fn cli_config_cli_overrides_toml_file() {
    let config_dir = tempfile::tempdir().unwrap();
    let cli_dir = tempfile::tempdir().unwrap();
    write_minimal_journal(cli_dir.path());
    let config_path = config_dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        format!(
            r#"
        [journal]
        folder = "{}"

        [monitor]
        live_status = true
        poll_interval_ms = 1500
        "#,
            config_dir.path().display()
        ),
    )
    .unwrap();

    let mut command = binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--journal")
        .arg(cli_dir.path())
        .arg("--poll-interval-ms")
        .arg("333")
        .arg("--no-status-line");
    let stdout = capture_watch_startup(command);
    assert!(stdout.contains(&format!("Journal folder: {}", cli_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

fn binary_command() -> std::process::Command {
    std::process::Command::new(assert_cmd::cargo::cargo_bin("ed-afk-dashboard"))
}

fn write_minimal_journal(folder: &std::path::Path) -> std::path::PathBuf {
    write_named_journal(folder, "Journal.2035-01-03T100000.01.log", "Minimal Raider")
}

fn write_named_journal(
    folder: &std::path::Path,
    filename: &str,
    ship_name: &str,
) -> std::path::PathBuf {
    let journal_path = folder.join(filename);
    std::fs::write(
        &journal_path,
        format!(
            concat!(
                r#"{{"timestamp":"2035-01-03T10:00:00Z","event":"Fileheader"}}"#,
                "\n",
                r#"{{"timestamp":"2035-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"{}","PilotName":"Fixture Pirate","LegalStatus":"Wanted"}}"#,
                "\n"
            ),
            ship_name
        ),
    )
    .unwrap();
    journal_path
}

fn capture_watch_startup(mut command: std::process::Command) -> String {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();

    std::thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch exited before it could tail Journal updates"
    );
    child.kill().ok();
    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_config_malformed_toml_exits_app_code_1() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("bad.toml");
    std::fs::write(&config_path, "[monitor\n").unwrap();

    let mut command = Command::cargo_bin("ed-afk-dashboard").unwrap();
    command
        .arg("--config")
        .arg(&config_path)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("malformed TOML config"));
}
