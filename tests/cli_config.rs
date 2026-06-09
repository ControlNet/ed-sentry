use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_config_help_lists_required_flags() {
    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();

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
        .stdout(predicate::str::contains("--no-status-line"));
}

#[test]
fn cli_config_bad_flag_exits_clap_code_2() {
    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();

    command
        .arg("--definitely-not-a-flag")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn cli_config_replay_rejects_poll_interval_ms_with_clap_code_2() {
    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();

    command
        .args([
            "replay",
            "--poll-interval-ms",
            "1000",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn cli_config_replay_requires_set_file() {
    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();

    command
        .arg("replay")
        .assert()
        .code(1)
        .stderr(predicate::str::contains(
            "replay requires --set-file <file>",
        ));
}

#[test]
fn cli_config_replay_rejects_journal() {
    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();

    command
        .args([
            "replay",
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
fn cli_config_global_flags_before_and_after_subcommands() {
    let mut before = Command::cargo_bin("ed-afk-watch").unwrap();
    before
        .args([
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "replay",
            "--no-status-line",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("replay configuration loaded"))
        .stdout(predicate::str::contains("live_status=false"));

    let mut after = Command::cargo_bin("ed-afk-watch").unwrap();
    after
        .args([
            "watch",
            "--journal",
            "/journals",
            "--poll-interval-ms",
            "222",
            "--no-status-line",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("journal='/journals'"))
        .stdout(predicate::str::contains("poll_interval_ms=222"))
        .stdout(predicate::str::contains("live_status=false"));
}

#[test]
fn cli_config_no_subcommand_watch_alias_binary() {
    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();

    command
        .args(["--journal", "/journals", "--no-status-line"])
        .assert()
        .success()
        .stdout(predicate::str::contains("watch configuration loaded"))
        .stdout(predicate::str::contains("journal='/journals'"))
        .stdout(predicate::str::contains("live_status=false"));
}

#[test]
fn cli_config_cli_overrides_toml_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
        [journal]
        folder = "/toml/journals"

        [monitor]
        live_status = true
        poll_interval_ms = 1500
        "#,
    )
    .unwrap();

    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();
    command
        .arg("--config")
        .arg(&config_path)
        .args([
            "watch",
            "--journal",
            "/cli/journals",
            "--poll-interval-ms",
            "333",
            "--no-status-line",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("journal='/cli/journals'"))
        .stdout(predicate::str::contains("poll_interval_ms=333"))
        .stdout(predicate::str::contains("live_status=false"));
}

#[test]
fn cli_config_malformed_toml_exits_app_code_1() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("bad.toml");
    std::fs::write(&config_path, "[monitor\n").unwrap();

    let mut command = Command::cargo_bin("ed-afk-watch").unwrap();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("watch")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("malformed TOML config"));
}
