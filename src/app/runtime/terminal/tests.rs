use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use tokio::sync::mpsc;

use super::*;
use crate::app::runtime::file_watcher::AfkWatcherEvent;
use crate::app::runtime::ConfiguredJournalSelector;
use crate::app::{AppLiveUpdate, AppSnapshot, ChecklistRowState};
use crate::config::{AppConfig, CliConfigOverrides, RuntimeConfig};
use crate::time::TimeDisplayZone;

#[tokio::test]
async fn terminal_watcher_selected_file_event_delivers_without_interval() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-06T100000.01.log");
    fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-06T10:00:00Z","event":"Commander","Name":"Cmdr Fixture"}"#,
            "\n",
            r#"{"timestamp":"2035-01-06T10:01:00Z","event":"Location","StarSystem":"Watcher System","Docked":false}"#,
            "\n"
        ),
    )
    .unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path.clone()),
        no_status_line: true,
        poll_interval_ms: Some(60_000),
        ..CliConfigOverrides::default()
    });
    let mut runtime = runtime_from_config(&config);
    runtime.process_preload(Utc.with_ymd_and_hms(2035, 1, 6, 10, 0, 0).unwrap());
    let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
    let mut delivery = DeliveryHub::terminal_only(terminal);

    append_line(
        &journal_path,
        r#"{"timestamp":"2035-01-06T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Viper Mk III","PilotName":"Watcher Raider","LegalStatus":"Wanted"}"#,
    );
    super::watch_loop::deliver_event(
        &mut runtime,
        &mut delivery,
        AfkWatcherEvent::SelectedFile { path: journal_path },
    )
    .await
    .unwrap();

    let output = String::from_utf8(delivery.into_terminal().into_inner()).unwrap();
    assert!(output.contains("Scan: Viper Mk III"), "{output}");
    println!("terminal_watcher selected_file_event=delivered_without_interval");
}

#[tokio::test]
async fn terminal_watcher_status_event_delivers_checklist_snapshot_only() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-06T110000.01.log");
    fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-06T11:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    let status_path = temp_dir.path().join("Status.json");
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path.clone()),
        no_status_line: true,
        ..CliConfigOverrides::default()
    });
    let mut runtime = runtime_from_config(&config);
    let subscriber = runtime.event_store().subscribe();
    let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
    let mut delivery = DeliveryHub::terminal_only(terminal);

    fs::write(&status_path, r#"{"Flags":64,"Pips":[4,0,8]}"#).unwrap();
    super::watch_loop::deliver_event(
        &mut runtime,
        &mut delivery,
        AfkWatcherEvent::StatusJson { path: status_path },
    )
    .await
    .unwrap();

    let output = String::from_utf8(delivery.into_terminal().into_inner()).unwrap();
    assert!(output.is_empty(), "{output}");
    assert_snapshot_update(subscriber.live.try_recv().unwrap());
    assert_eq!(
        checklist_state(
            &runtime.snapshot(Utc.with_ymd_and_hms(2035, 1, 6, 11, 1, 0).unwrap()),
            "hardpoints_deployed"
        ),
        ChecklistRowState::Pass
    );
    println!("terminal_watcher status_event=checklist_snapshot_only");
}

#[tokio::test]
async fn terminal_watcher_status_event_retries_partial_json_before_unknown() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-06T120000.01.log");
    fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-06T12:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    let status_path = temp_dir.path().join("Status.json");
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        no_status_line: true,
        ..CliConfigOverrides::default()
    });
    let mut runtime = runtime_from_config(&config);
    let subscriber = runtime.event_store().subscribe();
    let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
    let mut delivery = DeliveryHub::terminal_only(terminal);

    fs::write(&status_path, r#"{"Flags":"#).unwrap();
    let rewrite_path = status_path.clone();
    let rewrite = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        fs::write(&rewrite_path, r#"{"Flags":64,"Pips":[4,0,8]}"#).unwrap();
    });
    super::watch_loop::deliver_event(
        &mut runtime,
        &mut delivery,
        AfkWatcherEvent::StatusJson { path: status_path },
    )
    .await
    .unwrap();
    rewrite.await.unwrap();

    let output = String::from_utf8(delivery.into_terminal().into_inner()).unwrap();
    assert!(output.is_empty(), "{output}");
    assert_snapshot_update(recv_live_update(&subscriber.live).await);
    assert_eq!(
        checklist_state(
            &runtime.snapshot(Utc.with_ymd_and_hms(2035, 1, 6, 12, 1, 0).unwrap()),
            "hardpoints_deployed"
        ),
        ChecklistRowState::Pass
    );
    println!("terminal_watcher status_event=partial_retry_pass");
}

#[tokio::test]
async fn terminal_watcher_coalesces_duplicate_status_events_before_delivery() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-06T130000.01.log");
    fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-06T13:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    let status_path = temp_dir.path().join("Status.json");
    fs::write(&status_path, r#"{"Flags":64,"Pips":[4,0,8]}"#).unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        no_status_line: true,
        poll_interval_ms: Some(60_000),
        ..CliConfigOverrides::default()
    });
    let mut runtime = runtime_from_config(&config);
    let subscriber = runtime.event_store().subscribe();
    let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
    let mut delivery = DeliveryHub::terminal_only(terminal);
    let (sender, events) = mpsc::channel(4);
    let file_watcher = super::watch_loop::TerminalFileWatcher {
        _watcher: None,
        events: Some(events),
    };

    let producer_status = status_path.clone();
    let producer = tokio::spawn(async move {
        sender
            .send(AfkWatcherEvent::StatusJson {
                path: producer_status.clone(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        fs::write(&producer_status, r#"{"Flags":0,"Pips":[4,0,8]}"#).unwrap();
        sender
            .send(AfkWatcherEvent::StatusJson {
                path: producer_status,
            })
            .await
            .unwrap();
    });
    let run_result = tokio::time::timeout(
        Duration::from_millis(250),
        super::watch_loop::run_loop(&mut runtime, &mut delivery, file_watcher),
    )
    .await;
    producer.await.unwrap();

    assert!(run_result.is_err());
    let states = snapshot_states(&subscriber.live, "hardpoints_deployed");
    assert_eq!(
        states,
        vec![ChecklistRowState::Fail, ChecklistRowState::Fail]
    );
    println!("terminal_watcher duplicate_status_events=coalesced_final_state_only");
}

fn append_line(path: &Path, line: &str) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    writeln!(file, "{line}").unwrap();
}

fn runtime_from_config(config: &RuntimeConfig) -> MonitorRuntime {
    MonitorRuntime::start(
        config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap()
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

fn assert_snapshot_update(update: AppLiveUpdate) {
    match update {
        AppLiveUpdate::Snapshot { .. } => {}
        AppLiveUpdate::Event { item } => panic!("expected snapshot update, got event {item:?}"),
    }
}

async fn recv_live_update(receiver: &std::sync::mpsc::Receiver<AppLiveUpdate>) -> AppLiveUpdate {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            match receiver.try_recv() {
                Ok(update) => return update,
                Err(TryRecvError::Empty) => tokio::time::sleep(Duration::from_millis(5)).await,
                Err(TryRecvError::Disconnected) => panic!("event subscriber disconnected"),
            }
        }
    })
    .await
    .unwrap()
}

fn snapshot_states(
    receiver: &std::sync::mpsc::Receiver<AppLiveUpdate>,
    row_id: &str,
) -> Vec<ChecklistRowState> {
    receiver
        .try_iter()
        .filter_map(|update| match update {
            AppLiveUpdate::Snapshot { snapshot } => Some(checklist_state(&snapshot, row_id)),
            AppLiveUpdate::Event { .. } => None,
        })
        .collect()
}
