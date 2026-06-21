#[path = "cli_config/capture_output.rs"]
mod capture_output;
#[path = "cli_config/capture_text.rs"]
mod capture_text;
#[path = "cli_config/command.rs"]
mod command;
#[path = "cli_config/journal.rs"]
mod journal;
#[path = "cli_config/web.rs"]
mod web;

use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use std::time::Duration;

const WATCH_READINESS_DEADLINE: Duration = Duration::from_secs(10);

#[test]
fn cli_config_web_section_is_accepted_without_starting_server() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = journal::write_minimal_journal(temp_dir.path());
    let dist = tempfile::tempdir().unwrap();
    web::write_webui_dist(dist.path(), "accepted web section");
    let config_path = temp_dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        r#"
        [web]
        enabled = true
        host = "127.0.0.1"
        port = 0
        open_browser = false
        "#,
    )
    .unwrap();

    let mut command = command::binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line")
        .env("ED_SENTRY_WEBUI_DIST", dist.path());
    let stdout = capture_text::capture_watch_startup(command);

    assert!(stdout.contains("Scan: Minimal Raider"), "{stdout}");
    assert!(
        stdout.contains("Starting... (Press Ctrl+C to stop)"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Info: WebUI available at http://127.0.0.1:"),
        "{stdout}"
    );
}

#[test]
fn watch_webui_occupied_port_warns_and_continues_monitoring() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = journal::write_minimal_journal(temp_dir.path());
    let dist = tempfile::tempdir().unwrap();
    web::write_webui_dist(dist.path(), "watch occupied fixture");
    let occupied = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let config_path = temp_dir.path().join("config.toml");
    web::write_web_config(
        &config_path,
        temp_dir.path(),
        occupied.local_addr().unwrap().port(),
    );

    let mut command = command::binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--set-file")
        .arg(&journal_path)
        .arg("--no-status-line")
        .env("ED_SENTRY_WEBUI_DIST", dist.path());

    let output = capture_output::capture_watch_output_until(
        command,
        &["Info: WebUI unavailable", "Scan: Minimal Raider"],
        &["Warning: WebUI bind failed"],
        WATCH_READINESS_DEADLINE,
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.contains("Scan: Minimal Raider"), "{stdout}");
    assert!(stdout.contains("Info: WebUI unavailable"), "{stdout}");
    assert!(stderr.contains("Warning: WebUI bind failed"), "{stderr}");
}

#[test]
fn replay_does_not_start_webui_when_enabled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let dist = tempfile::tempdir().unwrap();
    web::write_webui_dist(dist.path(), "replay fixture");
    let occupied = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let config_path = temp_dir.path().join("config.toml");
    web::write_web_config(
        &config_path,
        temp_dir.path(),
        occupied.local_addr().unwrap().port(),
    );

    let mut command = command::binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--replay")
        .arg("--set-file")
        .arg("tests/fixtures/journal_combat_bounty.log")
        .arg("--no-status-line")
        .env("ED_SENTRY_WEBUI_DIST", dist.path());

    command
        .assert()
        .success()
        .stderr(predicate::str::contains("WebUI bind failed").not());
}
