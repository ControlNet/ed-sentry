use std::convert::Infallible;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Duration;

use ed_afk_dashboard::config::{AppConfig, CliConfigOverrides, LogLevelConfig, MonitorConfig};
use ed_afk_dashboard::event::parse_journal_line;
use ed_afk_dashboard::journal::{
    live_poll_interval, preload_journal_file, LiveTail, LiveTailWarning,
};
use ed_afk_dashboard::monitor::EventMonitor;
use ed_afk_dashboard::notifier::FakeNotifier;

#[test]
fn live_tail_no_preload_duplicate() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(&path, "preloaded-one\npreloaded-two\n").unwrap();
    let preload = preload_journal_file(&path, parse_line).unwrap();
    let mut tail = LiveTail::from_preload(&path, &preload);

    let first_poll = tail.poll(parse_line).unwrap();
    assert!(first_poll.records.is_empty());
    assert_eq!(first_poll.offset, preload.eof_offset);

    append_bytes(&path, b"live-one\n");
    let second_poll = tail.poll(parse_line).unwrap();

    assert_eq!(ok_lines(&second_poll), ["live-one"]);
    assert_eq!(second_poll.records[0].start_offset, preload.eof_offset);
    assert_eq!(tail.poll(parse_line).unwrap().records.len(), 0);
}

#[test]
fn live_tail_partial_line_waits_until_newline_and_processes_once() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(&path, "preloaded\n").unwrap();
    let preload = preload_journal_file(&path, parse_line).unwrap();
    let mut tail = LiveTail::from_preload(&path, &preload);

    append_bytes(&path, b"partial live");
    let partial_poll = tail.poll(parse_line).unwrap();

    assert!(partial_poll.records.is_empty());
    assert_eq!(tail.buffered_len(), "partial live".len());

    append_bytes(&path, b" complete\n");
    let complete_poll = tail.poll(parse_line).unwrap();

    assert_eq!(ok_lines(&complete_poll), ["partial live complete"]);
    assert_eq!(tail.buffered_len(), 0);
    assert!(tail.poll(parse_line).unwrap().records.is_empty());
}

#[test]
fn live_tail_handles_lf_and_crlf_line_endings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(&path, "").unwrap();
    let mut tail = LiveTail::from_offset(&path, 0);

    append_bytes(&path, b"lf-line\ncrlf-line\r\n");
    let poll = tail.poll(parse_line).unwrap();

    assert_eq!(ok_lines(&poll), ["lf-line", "crlf-line"]);
}

#[test]
fn live_tail_invalid_utf8_reports_error_and_keeps_offsets() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(&path, "").unwrap();
    let mut tail = LiveTail::from_offset(&path, 0);

    append_bytes(&path, b"ok\ninvalid-\xFF\nnext\n");
    let poll = tail.poll(parse_line).unwrap();

    assert_eq!(poll.records.len(), 3);
    assert_eq!(poll.records[0].result.as_ref().unwrap(), "ok");
    let error = poll.records[1].result.as_ref().unwrap_err();
    assert!(error.message.contains("invalid UTF-8"), "{}", error.message);
    assert_eq!(poll.records[2].result.as_ref().unwrap(), "next");
    assert_eq!(poll.offset, fs::metadata(&path).unwrap().len());
    assert_eq!(tail.buffered_len(), 0);
}

#[test]
fn live_tail_truncation_warns_and_resets_to_current_eof() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(&path, "preloaded\n").unwrap();
    let preload = preload_journal_file(&path, parse_line).unwrap();
    let mut tail = LiveTail::from_preload(&path, &preload);

    fs::write(&path, "cut\n").unwrap();
    let truncated_poll = tail.poll(parse_line).unwrap();

    assert!(truncated_poll.records.is_empty());
    assert_eq!(
        truncated_poll.warnings,
        [LiveTailWarning::FileTruncated {
            previous_offset: preload.eof_offset,
            new_offset: "cut\n".len() as u64,
        }]
    );
    assert_eq!(tail.offset(), "cut\n".len() as u64);

    append_bytes(&path, b"after-truncate\n");
    let next_poll = tail.poll(parse_line).unwrap();

    assert_eq!(ok_lines(&next_poll), ["after-truncate"]);
}

#[test]
fn live_tail_poll_interval_uses_runtime_config() {
    let mut config = AppConfig::default().into_runtime(&CliConfigOverrides::default());
    assert_eq!(live_poll_interval(&config), Duration::from_millis(1000));

    config.monitor = MonitorConfig {
        poll_interval_ms: 250,
        ..MonitorConfig::default()
    };
    assert_eq!(live_poll_interval(&config), Duration::from_millis(250));
}

#[test]
fn live_tail_temp_file_drives_monitor_notifier_pipeline_without_sleeping() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("Journal.2026-06-09T140000.01.log");
    fs::write(
        &path,
        concat!(
            r#"{"timestamp":"2035-07-01T10:00:00Z","event":"Commander","Name":"Cmdr Fixture Live"}"#,
            "\n",
            r#"{"timestamp":"2035-07-01T10:01:00Z","event":"Location","StarSystem":"Live Boundary System","Body":"Live Boundary Belt","Docked":false}"#,
            "\n"
        ),
    )
    .unwrap();
    let preload = preload_journal_file(&path, parse_journal_line).unwrap();
    let mut tail = LiveTail::from_preload(&path, &preload);
    let mut monitor = EventMonitor::new(
        FakeNotifier::new(),
        MonitorConfig::default(),
        LogLevelConfig::default(),
    );

    for record in &preload.records {
        monitor
            .process_event(record.result.as_ref().expect("preload event should parse"))
            .unwrap();
    }
    assert_eq!(
        monitor.state().system.as_deref(),
        Some("Live Boundary System")
    );
    assert!(monitor.dispatcher().notifier().notifications().is_empty());
    assert!(tail.poll(parse_journal_line).unwrap().records.is_empty());

    append_bytes(
        &path,
        br#"{"timestamp":"2035-07-01T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Viper Mk III","PilotName":"Fixture Live Raider","LegalStatus":"Wanted"}"#,
    );
    let partial_poll = tail.poll(parse_journal_line).unwrap();
    assert!(partial_poll.records.is_empty());
    assert!(monitor.dispatcher().notifier().notifications().is_empty());

    append_bytes(&path, b"\n");
    let scan_poll = tail.poll(parse_journal_line).unwrap();
    assert_eq!(scan_poll.records.len(), 1);
    monitor
        .process_event(scan_poll.records[0].result.as_ref().unwrap())
        .unwrap();

    let notifications = monitor.dispatcher().notifier().notifications();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].event_type, "ship_scan");
    assert!(notifications[0]
        .terminal_text
        .contains("Scan: Viper Mk III"));
    assert_eq!(monitor.state().cargo_scans, 0);

    append_bytes(
        &path,
        concat!(
            r#"{"timestamp":"2035-07-01T10:03:00Z","event":"Bounty","TotalReward":4200,"Target":"viper","VictimFaction":"Fixture Live Raiders"}"#,
            "\n"
        )
        .as_bytes(),
    );
    let kill_poll = tail.poll(parse_journal_line).unwrap();
    assert_eq!(kill_poll.records.len(), 1);
    monitor
        .process_event(kill_poll.records[0].result.as_ref().unwrap())
        .unwrap();

    let notifications = monitor.dispatcher().notifier().notifications();
    assert_eq!(notifications.len(), 2);
    assert_eq!(notifications[1].event_type, "kill_bounty");
    assert!(notifications[1].terminal_text.contains("Kill"));
    assert_eq!(monitor.state().kills, 1);
}

fn parse_line(line: &str) -> Result<String, Infallible> {
    Ok(line.to_string())
}

fn append_bytes(path: &std::path::Path, bytes: &[u8]) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}

fn ok_lines(poll: &ed_afk_dashboard::journal::LiveTailPoll<String>) -> Vec<&str> {
    poll.records
        .iter()
        .map(|record| record.result.as_ref().unwrap().as_str())
        .collect()
}
