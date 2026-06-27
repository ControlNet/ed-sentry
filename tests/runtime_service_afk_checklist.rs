use std::path::Path;

use chrono::{TimeZone, Utc};
use ed_sentry::app::runtime::{ConfiguredJournalSelector, MonitorRuntime};
use ed_sentry::app::{
    AppLiveUpdate, AppSnapshot, ChecklistRowState, MatrixStartupStatus, WebStartupStatus,
};
use ed_sentry::config::{AppConfig, CliConfigOverrides};

#[test]
fn afk_checklist_initializes_from_existing_companion_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-05T100000.01.log");
    std::fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-05T10:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    std::fs::write(
        temp_dir.path().join("Status.json"),
        r#"{"Flags":64,"Pips":[4,0,8]}"#,
    )
    .unwrap();
    std::fs::write(
        temp_dir.path().join("Cargo.json"),
        r#"{"Vessel":"Ship","Count":1}"#,
    )
    .unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        ..CliConfigOverrides::default()
    });

    let runtime = MonitorRuntime::start(
        &config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap();
    let snapshot = runtime.snapshot(Utc.with_ymd_and_hms(2035, 1, 5, 10, 1, 0).unwrap());

    assert_eq!(
        checklist_state(&snapshot, "hardpoints_deployed"),
        ChecklistRowState::Pass
    );
    assert_eq!(
        checklist_state(&snapshot, "engine_pips_zero"),
        ChecklistRowState::Pass
    );
    assert_eq!(
        checklist_state(&snapshot, "cargo_loaded"),
        ChecklistRowState::Pass
    );
    println!("startup_existing_files hardpoints=pass engine_pips=pass cargo=pass");
}

#[test]
fn afk_checklist_missing_companion_files_are_unknown() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-05T110000.01.log");
    std::fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-05T11:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        ..CliConfigOverrides::default()
    });

    let runtime = MonitorRuntime::start(
        &config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap();
    let snapshot = runtime.snapshot(Utc.with_ymd_and_hms(2035, 1, 5, 11, 1, 0).unwrap());

    assert_eq!(
        checklist_state(&snapshot, "hardpoints_deployed"),
        ChecklistRowState::Unknown
    );
    assert_eq!(
        checklist_state(&snapshot, "engine_pips_zero"),
        ChecklistRowState::Unknown
    );
    assert_eq!(
        checklist_state(&snapshot, "cargo_loaded"),
        ChecklistRowState::Unknown
    );
    println!("missing_companion_files hardpoints=unknown engine_pips=unknown cargo=unknown");
}

#[test]
fn afk_companion_update_publishes_checklist_only_snapshot() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-05T120000.01.log");
    write_journal(&journal_path, "2035-01-05T12:00:00Z");
    let mut runtime = start_runtime(&journal_path);
    let subscriber = runtime.event_store().subscribe();
    let status_path = temp_dir.path().join("Status.json");
    let cargo_path = temp_dir.path().join("Cargo.json");
    let now = Utc.with_ymd_and_hms(2035, 1, 5, 12, 1, 0).unwrap();

    std::fs::write(&status_path, r#"{"Flags":64,"Pips":[4,0,8]}"#).unwrap();
    let status_batch = runtime.process_companion_update(&status_path, now).unwrap();
    std::fs::write(&cargo_path, r#"{"Vessel":"Ship","Count":1}"#).unwrap();
    let cargo_batch = runtime.process_companion_update(&cargo_path, now).unwrap();

    assert!(status_batch.notifications.is_empty());
    assert!(cargo_batch.notifications.is_empty());
    assert!(status_batch.warnings.is_empty());
    assert!(cargo_batch.warnings.is_empty());
    assert_eq!(
        checklist_state(&cargo_batch.snapshot, "hardpoints_deployed"),
        ChecklistRowState::Pass
    );
    assert_eq!(
        checklist_state(&cargo_batch.snapshot, "engine_pips_zero"),
        ChecklistRowState::Pass
    );
    assert_eq!(
        checklist_state(&cargo_batch.snapshot, "cargo_loaded"),
        ChecklistRowState::Pass
    );
    assert_snapshot_update(subscriber.live.try_recv().unwrap());
    assert_snapshot_update(subscriber.live.try_recv().unwrap());
    assert!(subscriber.live.try_recv().is_err());
    println!("companion_update hardpoints=pass engine_pips=pass cargo=pass notifications=0");
}

#[test]
fn afk_cargo_journal_event_rereads_cargo_json_once() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-05T130000.01.log");
    write_journal(&journal_path, "2035-01-05T13:00:00Z");
    std::fs::write(
        temp_dir.path().join("Cargo.json"),
        r#"{"Vessel":"Ship","Count":0,"Inventory":[]}"#,
    )
    .unwrap();
    let mut runtime = start_runtime(&journal_path);
    std::fs::write(
        temp_dir.path().join("Cargo.json"),
        r#"{"Vessel":"Ship","Count":1}"#,
    )
    .unwrap();
    append_cargo_event(&journal_path, "2035-01-05T13:01:00Z");

    let batch = runtime
        .poll_once(Utc.with_ymd_and_hms(2035, 1, 5, 13, 1, 1).unwrap())
        .unwrap();

    assert!(batch.notifications.is_empty());
    assert!(batch.warnings.is_empty());
    assert_eq!(
        checklist_state(&batch.snapshot, "cargo_loaded"),
        ChecklistRowState::Pass
    );
    println!("cargo_journal_fallback cargo=pass notifications=0 warnings=0");
}

#[test]
fn afk_cargo_journal_event_malformed_cargo_json_sets_unknown_without_notifications() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-05T140000.01.log");
    write_journal(&journal_path, "2035-01-05T14:00:00Z");
    std::fs::write(
        temp_dir.path().join("Cargo.json"),
        r#"{"Vessel":"Ship","Count":1}"#,
    )
    .unwrap();
    let mut runtime = start_runtime(&journal_path);
    std::fs::write(
        temp_dir.path().join("Cargo.json"),
        r#"{"Vessel":"Ship","Count":"#,
    )
    .unwrap();
    append_cargo_event(&journal_path, "2035-01-05T14:01:00Z");

    let batch = runtime
        .poll_once(Utc.with_ymd_and_hms(2035, 1, 5, 14, 1, 1).unwrap())
        .unwrap();

    assert!(batch.notifications.is_empty());
    assert!(batch.warnings.is_empty());
    assert_eq!(
        checklist_state(&batch.snapshot, "cargo_loaded"),
        ChecklistRowState::Unknown
    );
    println!("cargo_malformed_fallback cargo=unknown notifications=0 warnings=0");
}

fn checklist_state(snapshot: &AppSnapshot, row_id: &str) -> ChecklistRowState {
    snapshot
        .afk_checklist
        .rows
        .iter()
        .find(|row| row.id == row_id)
        .unwrap()
        .state
}

fn write_journal(path: &Path, timestamp: &str) {
    std::fs::write(
        path,
        format!(
            r#"{{"timestamp":"{timestamp}","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}}"#
        ),
    )
    .unwrap();
}

fn append_cargo_event(path: &Path, timestamp: &str) {
    use std::io::Write;

    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    writeln!(
        file,
        r#"{{"timestamp":"{timestamp}","event":"Cargo","Vessel":"Ship","Count":1}}"#
    )
    .unwrap();
}

fn start_runtime(journal_path: &Path) -> MonitorRuntime {
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path.to_path_buf()),
        ..CliConfigOverrides::default()
    });

    MonitorRuntime::start(
        &config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap()
}

fn assert_snapshot_update(update: AppLiveUpdate) {
    match update {
        AppLiveUpdate::Snapshot { .. } => {}
        AppLiveUpdate::Event { item } => panic!("expected snapshot update, got event {item:?}"),
    }
}
