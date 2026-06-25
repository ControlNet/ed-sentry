use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_config_help_lists_required_flags() {
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

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
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

    command
        .arg("--definitely-not-a-flag")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn cli_config_replay_rejects_poll_interval_ms_as_no_effect() {
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

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
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

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
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

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
fn cli_config_debug_prints_runtime_diagnostics() {
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

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

    let output = Command::cargo_bin("ed-sentry-core")
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
fn cli_config_explicit_missing_config_still_errors() {
    let working_dir = tempfile::tempdir().unwrap();
    let missing_config = working_dir.path().join("missing.toml");
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

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
    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();

    command
        .current_dir(working_dir.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("malformed TOML config"));
}

#[test]
fn cli_config_malformed_toml_exits_app_code_1() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("bad.toml");
    std::fs::write(&config_path, "[monitor\n").unwrap();

    let mut command = Command::cargo_bin("ed-sentry-core").unwrap();
    command
        .arg("--config")
        .arg(&config_path)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("malformed TOML config"));
}
