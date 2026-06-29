use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex as StdMutex};

use chrono::{TimeZone, Utc};
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use super::*;
use crate::app::runtime::file_watcher::AfkWatcherEvent;
use crate::app::runtime::ConfiguredJournalSelector;
use crate::app::{AppLiveUpdate, ChecklistRowState, TunnelStatusKind};
use crate::config::{AppConfig, CliConfigOverrides};

#[tokio::test]
async fn desktop_watcher_selected_file_event_delivers_without_interval() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-08T100000.01.log");
    fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-08T10:00:00Z","event":"Commander","Name":"Cmdr Fixture"}"#,
            "\n",
            r#"{"timestamp":"2035-01-08T10:01:00Z","event":"Location","StarSystem":"Desktop Watcher","Docked":false}"#,
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
    let runtime = runtime_from_config(&config);
    runtime
        .lock()
        .await
        .process_preload(Utc.with_ymd_and_hms(2035, 1, 8, 10, 0, 0).unwrap());
    let delivery = memory_delivery();

    append_line(
        &journal_path,
        r#"{"timestamp":"2035-01-08T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Viper Mk III","PilotName":"Desktop Raider","LegalStatus":"Wanted"}"#,
    );
    deliver_desktop_event(
        &runtime,
        &delivery,
        AfkWatcherEvent::SelectedFile { path: journal_path },
    )
    .await
    .unwrap();

    let output = delivery_output(delivery).await;
    assert!(output.contains("Scan: Viper Mk III"), "{output}");
    println!("desktop_watcher selected_file_event=delivered_without_interval");
}

#[tokio::test]
async fn desktop_watcher_status_event_releases_runtime_before_delivery() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-08T110000.01.log");
    fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-08T11:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    let status_path = temp_dir.path().join("Status.json");
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        no_status_line: true,
        ..CliConfigOverrides::default()
    });
    let runtime = runtime_from_config(&config);
    let subscriber = runtime.lock().await.event_store().subscribe();
    let delivery = memory_delivery();
    let delivery_guard = delivery.lock().await;

    fs::write(&status_path, r#"{"Flags":"#).unwrap();
    let rewrite_path = status_path.clone();
    let rewrite = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(5)).await;
        fs::write(&rewrite_path, r#"{"Flags":64,"Pips":[4,0,8]}"#).unwrap();
    });
    let task_runtime = Arc::clone(&runtime);
    let task_delivery = Arc::clone(&delivery);
    let task = tokio::spawn(async move {
        deliver_desktop_event(
            &task_runtime,
            &task_delivery,
            AfkWatcherEvent::StatusJson { path: status_path },
        )
        .await
    });
    tokio::task::yield_now().await;
    drop(
        timeout(Duration::from_millis(20), runtime.lock())
            .await
            .unwrap(),
    );
    assert_snapshot_update(recv_live_update(&subscriber.live).await);
    rewrite.await.unwrap();

    let runtime_guard = timeout(Duration::from_millis(200), runtime.lock())
        .await
        .unwrap();
    assert_eq!(
        checklist_state(&runtime_guard, "hardpoints_deployed"),
        ChecklistRowState::Pass
    );
    drop(runtime_guard);
    drop(delivery_guard);
    task.await.unwrap().unwrap();
    assert!(delivery_output(delivery).await.is_empty());
    println!("desktop_watcher status_event=retry_without_runtime_lock_and_delivery_wait");
}

#[tokio::test]
async fn desktop_watcher_fallback_mode_keeps_polling() {
    let selected = std::env::temp_dir()
        .join(format!(
            "ed-sentry-missing-desktop-watch-parent-{}",
            std::process::id()
        ))
        .join("Journal.2035-01-08T120000.01.log");

    let file_watcher = DesktopFileWatcher::start(&selected);

    assert!(file_watcher.events.is_none());
    println!("desktop_watcher fallback_mode=polling_only");
}

#[tokio::test]
async fn desktop_gui_runtime_tunnel_commands_use_native_lifecycle_without_auth() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-08T123000.01.log");
    fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-08T12:30:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
    )
    .unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        no_status_line: true,
        ..CliConfigOverrides::default()
    });
    let runtime = runtime_from_config(&config);
    let event_store = runtime.lock().await.event_store();
    let desktop = DesktopRuntime {
        runtime: Arc::clone(&runtime),
        event_store,
        _web_server: WebServer::disabled(),
        tunnel: Some(WebTunnelState::new(None).unwrap()),
        monitor_task: tokio::spawn(async { std::future::pending::<()>().await }),
    };

    let initial = desktop.tunnel_status().await;
    let started = desktop.start_tunnel().await;

    assert_eq!(initial.kind, TunnelStatusKind::Disabled);
    assert_eq!(started.kind, TunnelStatusKind::Disabled);
    assert_eq!(
        started.message.as_deref(),
        Some("WebUI is not bound to a local port")
    );
    assert_eq!(
        runtime
            .lock()
            .await
            .snapshot(Utc.with_ymd_and_hms(2035, 1, 8, 12, 31, 0).unwrap())
            .tunnel
            .kind,
        TunnelStatusKind::Disabled
    );
    println!("desktop_runtime tunnel_commands=native_lifecycle_without_auth");
}

#[tokio::test]
async fn desktop_watcher_polling_fallback_uses_interval_cycle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-08T130000.01.log");
    fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-08T13:00:00Z","event":"Commander","Name":"Cmdr Fixture"}"#,
            "\n",
            r#"{"timestamp":"2035-01-08T13:01:00Z","event":"Location","StarSystem":"Desktop Fallback","Docked":false}"#,
            "\n"
        ),
    )
    .unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path.clone()),
        no_status_line: true,
        poll_interval_ms: Some(10),
        ..CliConfigOverrides::default()
    });
    let runtime = runtime_from_config(&config);
    runtime
        .lock()
        .await
        .process_preload(Utc.with_ymd_and_hms(2035, 1, 8, 13, 0, 0).unwrap());
    let buffer = SharedBuffer::default();
    let terminal = TerminalNotifier::plain(buffer.clone(), TimeDisplayZone::Utc);
    let delivery = Arc::new(Mutex::new(DeliveryHub::terminal_only(terminal)));
    let file_watcher = DesktopFileWatcher {
        _watcher: None,
        events: None,
    };

    append_line(
        &journal_path,
        r#"{"timestamp":"2035-01-08T13:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Viper Mk III","PilotName":"Fallback Raider","LegalStatus":"Wanted"}"#,
    );
    let task = tokio::spawn(run_monitor_loop(
        Arc::clone(&runtime),
        Arc::clone(&delivery),
        file_watcher,
    ));
    wait_for_output(&buffer, "Scan: Viper Mk III").await;

    task.abort();
    println!("desktop_watcher fallback_interval_poll=observed_output");
}

#[derive(Clone, Default)]
struct SharedBuffer {
    bytes: Arc<StdMutex<Vec<u8>>>,
}

impl SharedBuffer {
    fn contains(&self, needle: &str) -> bool {
        let bytes = self.bytes.lock().unwrap();
        String::from_utf8_lossy(&bytes).contains(needle)
    }
}

impl Write for SharedBuffer {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.bytes.lock().unwrap().write(buffer)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn runtime_from_config(config: &RuntimeConfig) -> Arc<Mutex<MonitorRuntime>> {
    Arc::new(Mutex::new(
        MonitorRuntime::start(
            config,
            &mut ConfiguredJournalSelector,
            MatrixStartupStatus::disabled(),
            WebStartupStatus::disabled(),
        )
        .unwrap(),
    ))
}

fn memory_delivery() -> Arc<Mutex<DeliveryHub<Vec<u8>>>> {
    let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
    Arc::new(Mutex::new(DeliveryHub::terminal_only(terminal)))
}

async fn delivery_output(delivery: Arc<Mutex<DeliveryHub<Vec<u8>>>>) -> String {
    let delivery = match Arc::try_unwrap(delivery) {
        Ok(delivery) => delivery.into_inner(),
        Err(_) => panic!("delivery still has multiple owners"),
    };
    String::from_utf8(delivery.into_terminal().into_inner()).unwrap()
}

async fn recv_live_update(receiver: &Receiver<AppLiveUpdate>) -> AppLiveUpdate {
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

async fn wait_for_output(buffer: &SharedBuffer, needle: &str) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if buffer.contains(needle) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}

fn append_line(path: &std::path::Path, line: &str) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    writeln!(file, "{line}").unwrap();
}

fn checklist_state(runtime: &MonitorRuntime, row_id: &str) -> ChecklistRowState {
    runtime
        .snapshot(Utc.with_ymd_and_hms(2035, 1, 8, 11, 1, 0).unwrap())
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
