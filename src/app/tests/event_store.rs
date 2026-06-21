use super::{empty_snapshot, timestamp};
use crate::app::runtime::{ConfiguredJournalSelector, MonitorRuntime};
use crate::app::{AppEventStore, AppLiveUpdate, MatrixStartupStatus, WebStartupStatus};
use crate::config::{AppConfig, CliConfigOverrides};
use crate::notifier::{AlertLevel, Notification};
use chrono::Duration;
use std::time::Duration as StdDuration;

pub(super) fn event_buffer() {
    let now = timestamp();
    let store = AppEventStore::with_capacity(empty_snapshot(now), 3);

    for index in 0..5 {
        store.record_lifecycle(
            "runtime_event",
            format!("runtime event {index}"),
            now + Duration::seconds(index),
        );
    }

    let subscriber = store.subscribe();
    let summaries: Vec<&str> = subscriber
        .bootstrap
        .recent_events
        .iter()
        .map(|item| item.summary.as_str())
        .collect();
    assert_eq!(
        summaries,
        vec!["runtime event 2", "runtime event 3", "runtime event 4"]
    );
}

pub(super) fn new_subscriber_receives_snapshot_and_recent_events() {
    let now = timestamp();
    let store = AppEventStore::with_capacity(empty_snapshot(now), 4);
    store.record_lifecycle("runtime_started", "Monitor started", now);
    let latest_snapshot = empty_snapshot(now + Duration::seconds(1));
    store.publish_snapshot(latest_snapshot.clone());

    let subscriber = store.subscribe();

    assert_eq!(
        subscriber.bootstrap.snapshot.generated_at,
        latest_snapshot.generated_at
    );
    assert_eq!(subscriber.bootstrap.snapshot.event_feed.len(), 1);
    assert_eq!(subscriber.bootstrap.recent_events.len(), 1);
    assert_eq!(
        subscriber.bootstrap.recent_events[0].summary,
        "Monitor started"
    );

    store.record_lifecycle(
        "runtime_poll",
        "Live event after subscription",
        now + Duration::seconds(2),
    );
    let update = subscriber
        .live
        .recv_timeout(StdDuration::from_secs(1))
        .unwrap();
    match update {
        AppLiveUpdate::Event { item } => {
            assert_eq!(item.summary, "Live event after subscription");
        }
        AppLiveUpdate::Snapshot { .. } => panic!("expected live event update"),
    }
}

pub(super) fn event_buffer_evicts_oldest_without_raw_journal_text() {
    let now = timestamp();
    let store = AppEventStore::with_capacity(empty_snapshot(now), 2);
    let raw_payload_seed =
        r#"raw_payload={"access_token":"seed-token","path":"/home/private/Journal.log"}"#;

    store.record_warning(raw_payload_seed, now);
    store.record_warning(
        "access_token=seed-token at /home/private/evicted-too.log",
        now + Duration::seconds(1),
    );
    store.record_warning(
        "Malformed journal line 2: invalid event timestamp",
        now + Duration::seconds(2),
    );
    store.record_warning(
        "Malformed journal line 3: invalid event timestamp",
        now + Duration::seconds(3),
    );

    let subscriber = store.subscribe();
    let summaries: Vec<&str> = subscriber
        .bootstrap
        .recent_events
        .iter()
        .map(|item| item.summary.as_str())
        .collect();
    let serialized = serde_json::to_string(&subscriber.bootstrap).unwrap();

    assert_eq!(
        summaries,
        vec![
            "Malformed journal line 2: invalid event timestamp",
            "Malformed journal line 3: invalid event timestamp"
        ]
    );
    assert!(!serialized.contains(raw_payload_seed));
    assert!(!serialized.contains("seed-token"));
    assert!(!serialized.contains("/home/private"));
}

pub(super) fn event_buffer_applies_line_safe_to_frontend_text() {
    let now = timestamp();
    let store = AppEventStore::with_capacity(empty_snapshot(now), 4);
    let notification = Notification::new(
        "ship\nscan",
        1,
        AlertLevel::Warn,
        Some("!\n".to_string()),
        "terminal text is not exposed here",
        "Target\nInjected\u{1b}[2K",
        now,
    );

    store.record_notification(&notification);

    let subscriber = store.subscribe();
    let event = &subscriber.bootstrap.recent_events[0];
    let notification = &subscriber.bootstrap.snapshot.notifications[0];
    assert_eq!(event.event_type, "ship scan");
    assert_eq!(event.summary, "Target Injected [2K");
    assert_eq!(notification.text, "Target Injected [2K");
    assert!(!event.summary.contains('\n'));
    assert!(!event.summary.contains('\u{1b}'));
    assert!(!notification.text.contains('\n'));
    assert!(!notification.text.contains('\u{1b}'));
}

#[test]
fn runtime_records_lifecycle_and_status_events_in_feed() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-03T100000.01.log");
    std::fs::write(
        &journal_path,
        r#"{"timestamp":"2035-01-03T10:00:00Z","event":"Commander","Name":"Cmdr Fixture"}"#,
    )
    .unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        ..CliConfigOverrides::default()
    });
    let started_at = timestamp();
    let status_at = started_at + Duration::seconds(1);

    let mut runtime = MonitorRuntime::start(
        &config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap();
    runtime.process_preload(started_at);

    let monitor_started = runtime.start_monitor_if_preloaded(status_at);
    assert!(monitor_started
        .snapshot
        .event_feed
        .iter()
        .any(|item| item.source == "lifecycle" && item.event_type == "monitor_started"));

    let status = runtime.status_snapshot(status_at, false);
    let status_line = status.status_line.as_deref().unwrap();
    assert!(status.snapshot.event_feed.iter().any(|item| {
        item.source == "status"
            && item.event_type == "runtime_status"
            && item.summary == status_line
    }));
}

#[test]
fn runtime_malformed_preload_warning_does_not_expose_raw_private_line() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-03T100000.01.log");
    let raw_line = r#"{"timestamp":"not-a-date","event":"Status","raw_payload":{"access_token":"seed-token","path":"/home/private/Journal.log"}}"#;
    std::fs::write(&journal_path, raw_line).unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path),
        ..CliConfigOverrides::default()
    });

    let mut runtime = MonitorRuntime::start(
        &config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap();
    let batch = runtime.process_preload(timestamp());
    let serialized = serde_json::to_string(&batch.snapshot).unwrap();

    assert!(batch
        .snapshot
        .event_feed
        .iter()
        .any(|item| item.source == "warning"));
    assert!(!serialized.contains(raw_line));
    assert!(!serialized.contains("seed-token"));
    assert!(!serialized.contains("/home/private"));
}
