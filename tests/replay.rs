use assert_cmd::Command;
use std::io::Write;

const ANSI_CLEAR_CURRENT_LINE: &str = "\u{1b}[2K";

#[test]
fn replay_combat_fixture_outputs_core_fragments() {
    let output = Command::cargo_bin("ed-afk-dashboard")
        .unwrap()
        .args([
            "--replay",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--no-status-line",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("ED AFK Dashboard v260421 by CMDR PSIPAB"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Journal file: journal_combat_bounty.log"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Commander name: Cmdr Fixture Bravo"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Starting... (Press Ctrl+C to stop)"),
        "{stdout}"
    );
    assert!(
        stdout.contains("[10:02:00]🔎 Scan: Viper Mk III (Competent)"),
        "{stdout}"
    );
    assert!(
        stdout.contains("[10:03:00]💥 Kill: Viper Mk III"),
        "{stdout}"
    );
    assert!(stdout.contains("[10:03:05]💥 Kill: Bond (+5s)"), "{stdout}");
    assert!(stdout.contains("Total Stats"), "{stdout}");
    assert!(stdout.contains("-> Kills: 2"), "{stdout}");
    assert!(stdout.contains("-> Bounties: 18k"), "{stdout}");
    assert!(stdout.contains("9k/kill"), "{stdout}");
    assert!(
        stdout.contains("Monitor stopped (journal_combat_bounty.log)"),
        "{stdout}"
    );
    assert!(!stdout.contains('\r'), "{stdout:?}");
    assert!(!stdout.contains(ANSI_CLEAR_CURRENT_LINE), "{stdout:?}");
}

#[test]
fn replay_malformed_fixture_warns_and_continues_to_summary() {
    let output = Command::cargo_bin("ed-afk-dashboard")
        .unwrap()
        .args([
            "--replay",
            "--set-file",
            "tests/fixtures/journal_malformed_unknown.log",
            "--no-status-line",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Malformed journal line"), "{stderr}");
    assert!(
        stdout.contains("Monitor stopped (journal_malformed_unknown.log)"),
        "{stdout}"
    );
    assert!(stdout.contains("Quit to desktop"), "{stdout}");
}

#[test]
fn replay_broad_events_stay_low_noise_and_malformed_lines_continue() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.broad-low-noise.log");
    std::fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"Commander","Name":"Cmdr Broad Fixture"}"#,
            "\n",
            r#"{"timestamp":"2035-01-09T10:01:00Z","event":"DockingGranted","LandingPad":42,"FixtureField":"quiet"}"#,
            "\n",
            "not-json\n",
            r#"{"timestamp":"2035-01-09T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Viper Mk III","LegalStatus":"Wanted"}"#,
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
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Malformed journal line"), "{stderr}");
    assert!(
        stdout.contains("Commander name: Cmdr Broad Fixture"),
        "{stdout}"
    );
    assert!(stdout.contains("Scan: Viper Mk III"), "{stdout}");
    assert!(!stdout.contains("DockingGranted"), "{stdout}");
    assert!(!stdout.contains("FixtureField"), "{stdout}");
}

#[test]
fn replay_reset_session_warning_is_printed_once() {
    let output = Command::cargo_bin("ed-afk-dashboard")
        .unwrap()
        .args([
            "--replay",
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--reset-session",
            "--no-status-line",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let warning_count = stderr
        .matches("--reset-session has no effect in replay")
        .count();
    assert_eq!(warning_count, 1, "{stderr}");
    assert!(stdout.contains("Total Stats"), "{stdout}");
}

#[test]
fn replay_config_output_options_are_observable() {
    let mut config = tempfile::NamedTempFile::new().unwrap();
    write!(
        config,
        r#"
        [monitor]
        pirate_names = false
        bounty_faction = false
        bounty_value = false
        extended_stats = false

        [log_levels]
        summary_kills = 1
        summary_scans = 0
        summary_bounties = 0
        summary_faction = 0
        "#
    )
    .unwrap();

    let output = Command::cargo_bin("ed-afk-dashboard")
        .unwrap()
        .args([
            "--replay",
            "--config",
            config.path().to_str().unwrap(),
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--no-status-line",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("Scan: Viper Mk III (Competent)"),
        "{stdout}"
    );
    assert!(stdout.contains("Kill: Viper Mk III\n"), "{stdout}");
    assert!(stdout.contains("Kill: Bond (+5s)\n"), "{stdout}");
    assert!(stdout.contains("Total Stats"), "{stdout}");
    assert!(!stdout.contains("Fixture Raider One"), "{stdout}");
    assert!(!stdout.contains("Practice Raiders"), "{stdout}");
    assert!(!stdout.contains("6400 cr"), "{stdout}");
    assert!(!stdout.contains("-> Bounties"), "{stdout}");
}

#[test]
fn replay_summary_log_levels_control_summary_fragments() {
    let mut config = tempfile::NamedTempFile::new().unwrap();
    write!(
        config,
        r#"
        [monitor]
        extended_stats = false

        [log_levels]
        summary_kills = 0
        summary_scans = 1
        summary_bounties = 1
        summary_faction = 1
        "#
    )
    .unwrap();

    let output = Command::cargo_bin("ed-afk-dashboard")
        .unwrap()
        .args([
            "--replay",
            "--config",
            config.path().to_str().unwrap(),
            "--set-file",
            "tests/fixtures/journal_combat_bounty.log",
            "--no-status-line",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let summary = stdout
        .lines()
        .find(|line| line.contains("Total Stats"))
        .unwrap();
    assert!(!summary.contains("Kills"), "{summary}");
    assert!(stdout.contains("-> Bounties: 18k"), "{stdout}");
    assert!(!stdout.contains("-> Scans:"), "{stdout}");
    assert!(stdout.contains("-> Faction: 2"), "{stdout}");
}

#[test]
fn replay_does_not_emit_live_idle_warnings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.warning.log");
    std::fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"SupercruiseDestinationDrop","Type":"ResourceExtraction","Type_Localised":"Resource Extraction Site"}"#,
            "\n",
            r#"{"timestamp":"2035-01-09T10:01:00Z","event":"Bounty","TotalReward":4200,"Target":"viper","VictimFaction":"Fixture Raiders"}"#,
            "\n",
            r#"{"timestamp":"2035-01-09T10:06:00Z","event":"Fileheader"}"#,
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
    assert!(!stdout.contains("Kill rate of"), "{stdout}");
    assert!(!stdout.contains("No kills logged"), "{stdout}");
}
