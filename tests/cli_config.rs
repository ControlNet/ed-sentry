use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;

#[test]
fn cli_config_help_lists_required_flags() {
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

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
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

    command
        .arg("--definitely-not-a-flag")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn cli_config_replay_rejects_poll_interval_ms_as_no_effect() {
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

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
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

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
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

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
    let mut before = Command::cargo_bin("ed-sentry").unwrap();
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
            "ed-sentry v260421 by CMDR ControlNet",
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
    assert!(stdout.contains("ed-sentry v260421 by CMDR ControlNet"));
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
fn watch_matrix_init_failure_falls_back_to_terminal() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = write_minimal_journal(temp_dir.path());
    let config_path = temp_dir.path().join("config.toml");
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line")
        .env("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG", &matrix_log)
        .env(
            "ED_AFK_DASHBOARD_FAKE_MATRIX_CONNECT_ERROR",
            "connect refused fixture-access",
        );

    let output = capture_watch_startup_output(command, Duration::from_millis(250));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(
        stdout.contains("Info: Matrix delivery unavailable"),
        "{stdout}"
    );
    assert!(stdout.contains("Scan: Minimal Raider"), "{stdout}");
    assert!(
        stderr.contains("Warning: Matrix delivery disabled:"),
        "{stderr}"
    );
    assert!(stderr.contains("connect refused <redacted>"), "{stderr}");
    assert!(!stderr.contains("fixture-access"), "{stderr}");
    assert!(!matrix_log.exists() || std::fs::read_to_string(&matrix_log).unwrap().is_empty());
}

#[test]
fn watch_level_one_and_two_notifications_reach_fake_matrix() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-03T100000.01.log");
    std::fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2020-01-03T10:00:00Z","event":"Fileheader"}"#,
            "\n",
            r#"{"timestamp":"2020-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"History Viper","PilotName":"Fixture Pirate","LegalStatus":"Wanted"}"#,
            "\n",
            r#"{"timestamp":"2020-01-03T10:03:00Z","event":"Bounty","TotalReward":6400,"Target":"viper","Target_Localised":"History Viper","VictimFaction":"Practice Raiders"}"#,
            "\n"
        ),
    )
    .unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .env("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG", &matrix_log);

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    std::thread::sleep(Duration::from_millis(200));
    append_journal_lines(
        &journal_path,
        concat!(
            r#"{"timestamp":"2099-01-03T10:04:00Z","event":"ReservoirReplenished","FuelMain":8.0,"FuelReservoir":0.63}"#,
            "\n",
            r#"{"timestamp":"2099-01-03T10:05:00Z","event":"Bounty","TotalReward":6400,"Target":"cobra","Target_Localised":"Live Cobra","VictimFaction":"Practice Raiders"}"#,
            "\n"
        ),
    );
    std::thread::sleep(Duration::from_millis(1200));
    child.kill().ok();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("Info: Matrix delivery enabled"), "{stdout}");
    assert!(stdout.contains("Scan: History Viper"), "{stdout}");
    assert!(stdout.contains("Kill: History Viper"), "{stdout}");
    assert!(stdout.contains("Fuel:"), "{stdout}");
    assert!(stdout.contains("Kill: Live Cobra"), "{stdout}");
    assert!(!stdout.contains("📦"), "{stdout}");
    assert!(!stdout.contains("⏱️"), "{stdout}");

    let records = read_matrix_records(&matrix_log);
    assert!(
        records.iter().any(|record| record["kind"] == "connect"),
        "{records:?}"
    );
    let sends = records
        .iter()
        .filter(|record| record["kind"] == "send")
        .collect::<Vec<_>>();
    assert!(
        sends.iter().any(|record| {
            let remote_text = record["remote_text"].as_str().unwrap();
            record["event_type"] == "matrix_startup"
                && remote_text.starts_with("🛰️ ed-sentry started\nVersion: ")
                && remote_text.contains("\nStarted at:")
                && remote_text.contains("\nJournal folder:")
                && remote_text.contains("\nJournal file: Journal.2035-01-03T100000.01.log")
                && remote_text.contains("\nMatrix room: !room:matrix.invalid")
                && !remote_text.contains(" | ")
        }),
        "{sends:?}"
    );
    assert!(
        sends.iter().all(|record| !record["remote_text"]
            .as_str()
            .unwrap()
            .contains("History Viper")),
        "{sends:?}"
    );
    assert!(
        sends.iter().any(|record| {
            record["level"] == 1
                && record["mention"] == false
                && record["remote_text"]
                    .as_str()
                    .unwrap()
                    .contains("Kill: Live Cobra")
        }),
        "{sends:?}"
    );
    assert!(
        sends.iter().any(|record| {
            record["level"] == 2
                && record["mention"] == true
                && record["remote_text"].as_str().unwrap().contains("Fuel:")
        }),
        "{sends:?}"
    );
    assert!(
        records
            .iter()
            .any(|record| { record["kind"] == "status" && record["force"] == true }),
        "{records:?}"
    );
}

#[test]
fn watch_plain_stdout_does_not_render_live_status_but_matrix_status_publishes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = write_minimal_journal(temp_dir.path());
    let config_path = temp_dir.path().join("config.toml");
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .env("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG", &matrix_log);

    let output = capture_watch_startup_output(command, Duration::from_millis(250));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("Info: Matrix delivery enabled"), "{stdout}");
    assert!(stdout.contains("Scan: Minimal Raider"), "{stdout}");
    assert!(!stdout.contains("📦"), "{stdout}");
    assert!(!stdout.contains("⏱️"), "{stdout}");

    let records = read_matrix_records(&matrix_log);
    assert!(
        records
            .iter()
            .any(|record| record["kind"] == "status" && record["force"] == true),
        "{records:?}"
    );
}

#[test]
fn watch_delayed_matrix_send_warns_without_blocking_terminal_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = write_minimal_journal(temp_dir.path());
    let config_path = temp_dir.path().join("config.toml");
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line")
        .env("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG", &matrix_log)
        .env("ED_AFK_DASHBOARD_FAKE_MATRIX_SEND_DELAY_MS", "25")
        .env(
            "ED_AFK_DASHBOARD_FAKE_MATRIX_SEND_ERROR",
            "delayed remote failure",
        );

    let output = capture_watch_startup_output(command, Duration::from_millis(300));
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("Scan: Minimal Raider"), "{stdout}");
    assert!(stdout.contains("Info: Matrix delivery enabled"), "{stdout}");
    assert!(
        stderr.contains("Warning: delayed remote failure"),
        "{stderr}"
    );
    let records = read_matrix_records(&matrix_log);
    assert!(
        records.iter().all(|record| record["kind"] != "status"),
        "{records:?}"
    );
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
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

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

    let output = Command::cargo_bin("ed-sentry")
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

#[test]
fn cli_config_implicit_config_toml_absent_uses_defaults() {
    let working_dir = tempfile::tempdir().unwrap();
    let journal_dir = tempfile::tempdir().unwrap();
    write_minimal_journal(journal_dir.path());

    let mut command = binary_command();
    command
        .current_dir(working_dir.path())
        .arg("--journal")
        .arg(journal_dir.path())
        .arg("--no-status-line");
    let stdout = capture_watch_startup(command);

    assert!(stdout.contains(&format!("Journal folder: {}", journal_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

#[test]
fn cli_config_implicit_config_toml_loads_when_config_flag_absent() {
    let working_dir = tempfile::tempdir().unwrap();
    let journal_dir = tempfile::tempdir().unwrap();
    write_minimal_journal(journal_dir.path());
    std::fs::write(
        working_dir.path().join("config.toml"),
        format!(
            r#"
            [journal]
            folder = "{}"

            [monitor]
            poll_interval_ms = 200
            "#,
            journal_dir.path().display()
        ),
    )
    .unwrap();

    let mut command = binary_command();
    command
        .current_dir(working_dir.path())
        .arg("--no-status-line");
    let stdout = capture_watch_startup(command);

    assert!(stdout.contains(&format!("Journal folder: {}", journal_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

#[test]
fn cli_config_explicit_missing_config_still_errors() {
    let working_dir = tempfile::tempdir().unwrap();
    let missing_config = working_dir.path().join("missing.toml");
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

    command
        .current_dir(working_dir.path())
        .arg("--config")
        .arg(&missing_config)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("failed to read config"));
}

#[test]
fn cli_config_implicit_malformed_config_toml_errors() {
    let working_dir = tempfile::tempdir().unwrap();
    std::fs::write(working_dir.path().join("config.toml"), "[monitor\n").unwrap();
    let mut command = Command::cargo_bin("ed-sentry").unwrap();

    command
        .current_dir(working_dir.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("malformed TOML config"));
}

fn binary_command() -> std::process::Command {
    std::process::Command::new(assert_cmd::cargo::cargo_bin("ed-sentry"))
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

fn append_journal_lines(path: &std::path::Path, lines: &str) {
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(lines.as_bytes()).unwrap();
    file.flush().unwrap();
}

fn capture_watch_startup(command: std::process::Command) -> String {
    let output = capture_watch_startup_output(command, Duration::from_millis(150));
    String::from_utf8(output.stdout).unwrap()
}

fn capture_watch_startup_output(
    mut command: std::process::Command,
    wait: Duration,
) -> std::process::Output {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();

    std::thread::sleep(wait);
    assert!(
        child.try_wait().unwrap().is_none(),
        "watch exited before it could tail Journal updates"
    );
    child.kill().ok();
    child.wait_with_output().unwrap()
}

fn write_matrix_config(
    path: &std::path::Path,
    journal_folder: &std::path::Path,
    live_status: bool,
) {
    std::fs::write(
        path,
        format!(
            r#"
            [journal]
            folder = "{}"

            [monitor]
            live_status = {}
            poll_interval_ms = 1000

            [matrix]
            enabled = true
            homeserver = "https://matrix.invalid"
            user_id = "@bot:matrix.invalid"
            room_id = "!room:matrix.invalid"
            access_{} = "fixture-access"
            mention_user_id = "@commander:matrix.invalid"
            status_update_interval_seconds = 60
            "#,
            journal_folder.display(),
            live_status,
            "token",
        ),
    )
    .unwrap();
}

fn read_matrix_records(path: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn cli_config_malformed_toml_exits_app_code_1() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("bad.toml");
    std::fs::write(&config_path, "[monitor\n").unwrap();

    let mut command = Command::cargo_bin("ed-sentry").unwrap();
    command
        .arg("--config")
        .arg(&config_path)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("malformed TOML config"));
}
