#[path = "cli_config/capture_output.rs"]
mod capture_output;
#[path = "cli_config/command.rs"]
mod command;
#[path = "cli_config/journal.rs"]
mod journal;
#[path = "cli_config/matrix.rs"]
mod matrix;

use std::time::Duration;

const WATCH_READINESS_DEADLINE: Duration = Duration::from_secs(10);

#[test]
fn watch_matrix_init_failure_falls_back_to_terminal() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = journal::write_minimal_journal(temp_dir.path());
    let config_path = temp_dir.path().join("config.toml");
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    matrix::write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = command::binary_command();
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

    let output = capture_output::capture_watch_output_until(
        command,
        &["Info: Matrix delivery unavailable", "Scan: Minimal Raider"],
        &["Warning: Matrix delivery disabled:"],
        WATCH_READINESS_DEADLINE,
    );
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
    matrix::write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = command::binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .env("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG", &matrix_log);

    let mut watch = capture_output::RunningWatch::spawn(command);
    watch.wait_for_output(
        &[
            "Info: Matrix delivery enabled",
            "Scan: History Viper",
            "Kill: History Viper",
        ],
        &[],
        WATCH_READINESS_DEADLINE,
    );
    matrix::append_journal_lines(
        &journal_path,
        concat!(
            r#"{"timestamp":"2099-01-03T10:04:00Z","event":"ReservoirReplenished","FuelMain":8.0,"FuelReservoir":0.63}"#,
            "\n",
            r#"{"timestamp":"2099-01-03T10:05:00Z","event":"Bounty","TotalReward":6400,"Target":"cobra","Target_Localised":"Live Cobra","VictimFaction":"Practice Raiders"}"#,
            "\n"
        ),
    );
    watch.wait_for_output(
        &["Fuel:", "Kill: Live Cobra"],
        &[],
        WATCH_READINESS_DEADLINE,
    );
    let output = watch.stop();
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

    let records = matrix::wait_for_matrix_record(
        &matrix_log,
        matrix::is_live_cobra_send_record,
        WATCH_READINESS_DEADLINE,
    );
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
    let journal_path = journal::write_minimal_journal(temp_dir.path());
    let config_path = temp_dir.path().join("config.toml");
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    matrix::write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = command::binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .env("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG", &matrix_log);

    let mut watch = capture_output::RunningWatch::spawn(command);
    watch.wait_for_output(
        &["Info: Matrix delivery enabled", "Scan: Minimal Raider"],
        &[],
        WATCH_READINESS_DEADLINE,
    );
    let records = matrix::wait_for_matrix_record(
        &matrix_log,
        is_forced_status_record,
        WATCH_READINESS_DEADLINE,
    );
    let output = watch.stop();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stderr.is_empty(), "{stderr}");
    assert!(stdout.contains("Info: Matrix delivery enabled"), "{stdout}");
    assert!(stdout.contains("Scan: Minimal Raider"), "{stdout}");
    assert!(!stdout.contains("📦"), "{stdout}");
    assert!(!stdout.contains("⏱️"), "{stdout}");

    assert!(
        records
            .iter()
            .any(|record| record["kind"] == "status" && record["force"] == true),
        "{records:?}"
    );
}

fn is_forced_status_record(record: &serde_json::Value) -> bool {
    record["kind"] == "status" && record["force"] == true
}

#[test]
fn watch_delayed_matrix_send_warns_without_blocking_terminal_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = journal::write_minimal_journal(temp_dir.path());
    let config_path = temp_dir.path().join("config.toml");
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    matrix::write_matrix_config(&config_path, temp_dir.path(), true);

    let mut command = command::binary_command();
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

    let output = capture_output::capture_watch_output_until(
        command,
        &["Scan: Minimal Raider", "Info: Matrix delivery enabled"],
        &["Warning: delayed remote failure"],
        WATCH_READINESS_DEADLINE,
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("Scan: Minimal Raider"), "{stdout}");
    assert!(stdout.contains("Info: Matrix delivery enabled"), "{stdout}");
    assert!(
        stderr.contains("Warning: delayed remote failure"),
        "{stderr}"
    );
    let records = matrix::read_matrix_records(&matrix_log);
    assert!(
        records.iter().all(|record| record["kind"] != "status"),
        "{records:?}"
    );
}
