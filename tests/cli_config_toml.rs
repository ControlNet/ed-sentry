#[path = "cli_config/capture_text.rs"]
mod capture_text;
#[path = "cli_config/command.rs"]
mod command;
#[path = "cli_config/journal.rs"]
mod journal;

#[test]
fn cli_config_cli_overrides_toml_file() {
    let config_dir = tempfile::tempdir().unwrap();
    let cli_dir = tempfile::tempdir().unwrap();
    journal::write_minimal_journal(cli_dir.path());
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

    let mut command = command::binary_command();
    command
        .arg("--config")
        .arg(&config_path)
        .arg("--journal")
        .arg(cli_dir.path())
        .arg("--poll-interval-ms")
        .arg("333")
        .arg("--no-status-line");
    let stdout = capture_text::capture_watch_startup(command);
    assert!(stdout.contains(&format!("Journal folder: {}", cli_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

#[test]
fn cli_config_implicit_config_toml_absent_uses_defaults() {
    let working_dir = tempfile::tempdir().unwrap();
    let journal_dir = tempfile::tempdir().unwrap();
    journal::write_minimal_journal(journal_dir.path());

    let mut command = command::binary_command();
    command
        .current_dir(working_dir.path())
        .arg("--journal")
        .arg(journal_dir.path())
        .arg("--no-status-line");
    let stdout = capture_text::capture_watch_startup(command);

    assert!(stdout.contains(&format!("Journal folder: {}", journal_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}

#[test]
fn cli_config_implicit_config_toml_loads_when_config_flag_absent() {
    let working_dir = tempfile::tempdir().unwrap();
    let journal_dir = tempfile::tempdir().unwrap();
    journal::write_minimal_journal(journal_dir.path());
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

    let mut command = command::binary_command();
    command
        .current_dir(working_dir.path())
        .arg("--no-status-line");
    let stdout = capture_text::capture_watch_startup(command);

    assert!(stdout.contains(&format!("Journal folder: {}", journal_dir.path().display())));
    assert!(stdout.contains("Starting... (Press Ctrl+C to stop)"));
}
