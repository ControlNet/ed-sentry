use super::*;
use crate::config::{AppConfig, CliConfigOverrides, MatrixConfig};
use crate::event::{BasicJournalEvent, JournalEvent, MissionEvent};
use crate::mission::MissionTracker;
use crate::state::SessionState;
use chrono::{Duration, TimeZone, Utc};
use std::path::PathBuf;

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

mod event_store;

fn timestamp() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 9, 15, 30, 0)
        .single()
        .unwrap()
}

fn accepted_massacre_mission(timestamp: chrono::DateTime<Utc>) -> JournalEvent {
    JournalEvent::MissionAccepted(MissionEvent {
        timestamp,
        event: "MissionAccepted".to_string(),
        raw: None,
        mission_id: Some(42),
        name: Some("Mission_Massacre".to_string()),
        localised_name: Some("Massacre pirates".to_string()),
        faction: Some("Blue Crew".to_string()),
        target_faction: Some("Red Crew".to_string()),
        target: Some("pirates".to_string()),
        target_type: Some("Pirate".to_string()),
        destination_system: Some("Odelite".to_string()),
        destination_station: None,
        destination_settlement: None,
        new_destination_system: None,
        new_destination_station: None,
        old_destination_system: None,
        old_destination_station: None,
        expiry: Some(timestamp + Duration::hours(6)),
        influence: Some("Med".to_string()),
        reputation: Some("High".to_string()),
        reward: Some(1_250_000),
        donated: None,
        fine: None,
        wing: Some(false),
        commodity: None,
        commodity_localised: None,
        count: None,
        kill_count: Some(12),
    })
}

pub(super) fn empty_snapshot(now: chrono::DateTime<Utc>) -> AppSnapshot {
    AppSnapshot::from_state(
        &SessionState::new(),
        &MissionTracker::new(),
        now,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
}

#[test]
fn app_snapshot_includes_session_mission_and_display_values() {
    let now = timestamp();
    let mut state = SessionState::new();
    state.commander = Some("CMDR Fixture".to_string());
    state.ship = Some("Type-10 Defender".to_string());
    state.system = Some("Odelite".to_string());
    state.shields_up = Some(true);
    state.ship_hull = Some(0.875);
    state.start_session_at(now - Duration::hours(2));
    state.record_incoming_scan(now - Duration::minutes(20));
    state.record_incoming_scan(now - Duration::minutes(5));

    let mut tracker = MissionTracker::new();
    tracker.apply_event(&accepted_massacre_mission(now - Duration::minutes(10)));

    let snapshot = AppSnapshot::from_state(
        &state,
        &tracker,
        now,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    );

    assert_eq!(snapshot.session.commander.as_deref(), Some("CMDR Fixture"));
    assert_eq!(snapshot.session.ship.as_deref(), Some("Type-10 Defender"));
    assert_eq!(snapshot.session.system.as_deref(), Some("Odelite"));
    assert_eq!(snapshot.session.elapsed_seconds, 7_200);
    assert_eq!(snapshot.session.elapsed_display, "2h0m");
    assert_eq!(snapshot.session.scan_total_rate_per_hour.value, 1.0);
    assert_eq!(snapshot.session.scan_total_rate_per_hour.display, "1.0/h");
    assert_eq!(snapshot.session.bounty_total.display, "0 cr");
    assert_eq!(snapshot.session.status_label, "Active");
    assert_eq!(snapshot.missions.active_count, 1);
    assert_eq!(
        snapshot.missions.items[0].display_name,
        "Massacre pirates".to_string()
    );
    assert_eq!(snapshot.missions.items[0].reward.display, "1,250,000 cr");
    assert_eq!(snapshot.matrix.status_label, "Disabled");
    assert_eq!(snapshot.web.status_label, "Disabled");
}

#[test]
fn config_view_redacts_matrix_access_token() {
    let config = AppConfig {
        matrix: Some(MatrixConfig {
            enabled: true,
            homeserver: Some("https://matrix.fixture.invalid".to_string()),
            room_id: Some("!room:matrix.fixture.invalid".to_string()),
            access_token: Some("redaction-fixture-value".to_string()),
            mention_user_id: Some("@mention:matrix.fixture.invalid".to_string()),
            status_update_interval_seconds: 90,
        }),
        ..AppConfig::default()
    };
    let runtime = config.into_runtime(&CliConfigOverrides {
        set_file: Some(PathBuf::from("Journal.fixture.log")),
        ..CliConfigOverrides::default()
    });

    let view = EditableConfigView::from_runtime_config(&runtime);
    let json = serde_json::to_string(&view).unwrap();

    let matrix = view.matrix.unwrap();
    assert!(matrix.access_token_present);
    assert!(json.contains("\"access_token_present\":true"));
    assert!(!json.contains("redaction-fixture-value"));
    assert!(!json.contains("access_token\":\""));
}

#[test]
fn session_view_uses_ended_at_for_elapsed_display() {
    let started_at = timestamp() - Duration::hours(2);
    let ended_at = started_at + Duration::minutes(30);
    let now = started_at + Duration::hours(3);
    let mut state = SessionState::new();
    state.start_session_at(started_at);
    state.apply_event(&JournalEvent::Shutdown(BasicJournalEvent {
        timestamp: ended_at,
        event: "Shutdown".to_string(),
        raw: None,
    }));

    let view = SessionView::from_state(&state, now);

    assert!(!view.active);
    assert_eq!(view.status_label, "Ended");
    assert_eq!(view.elapsed_seconds, 1_800);
    assert_eq!(view.elapsed_display, "30m");
    assert_eq!(view.ended_at, Some(ended_at));
}

#[test]
fn event_buffer() {
    event_store::event_buffer();
}

#[test]
fn new_subscriber_receives_snapshot_and_recent_events() {
    event_store::new_subscriber_receives_snapshot_and_recent_events();
}

#[test]
fn event_buffer_evicts_oldest_without_raw_journal_text() {
    event_store::event_buffer_evicts_oldest_without_raw_journal_text();
}

#[test]
fn event_buffer_applies_line_safe_to_frontend_text() {
    event_store::event_buffer_applies_line_safe_to_frontend_text();
}

#[tokio::test]
async fn desktop_bootstrap_honors_web_and_matrix_config() {
    let _guard = ENV_LOCK.lock().await;
    let temp_dir = tempfile::tempdir().unwrap();
    let dist_dir = temp_dir.path().join("dist");
    std::fs::create_dir_all(&dist_dir).unwrap();
    std::fs::write(
        dist_dir.join("index.html"),
        "<!doctype html><title>ed-sentry</title><main>desktop bootstrap</main>",
    )
    .unwrap();
    let matrix_log = temp_dir.path().join("matrix.jsonl");
    std::env::set_var("ED_SENTRY_WEBUI_DIST", &dist_dir);
    std::env::set_var("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG", &matrix_log);

    let journal_dir = temp_dir.path().join("journal");
    std::fs::create_dir_all(&journal_dir).unwrap();
    let journal_file = journal_dir.join("Journal.2035-01-03T100000.01.log");
    std::fs::write(
        &journal_file,
        concat!(
            r#"{"timestamp":"2035-01-03T10:00:00Z","event":"Commander","Name":"Desktop Fixture"}"#,
            "\n",
            r#"{"timestamp":"2035-01-03T10:02:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Desktop Raider","PilotName":"Fixture Pirate","LegalStatus":"Wanted"}"#,
            "\n"
        ),
    )
    .unwrap();
    let runtime = AppConfig {
        journal: crate::config::JournalConfig {
            folder: journal_dir.to_string_lossy().into_owned(),
            recent_files: 5,
        },
        matrix: Some(MatrixConfig {
            enabled: true,
            homeserver: Some("https://matrix.fixture.invalid".to_string()),
            room_id: Some("!desktop-room:matrix.fixture.invalid".to_string()),
            access_token: Some("desktop-fixture-token".to_string()),
            mention_user_id: Some("@mention:matrix.fixture.invalid".to_string()),
            status_update_interval_seconds: 60,
        }),
        web: crate::config::WebConfig {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 0,
            open_browser: false,
        },
        ..AppConfig::default()
    }
    .into_runtime(&CliConfigOverrides {
        set_file: Some(journal_file),
        ..CliConfigOverrides::default()
    });

    let runtime_for_view = runtime.clone();
    let desktop = runtime::DesktopRuntime::start(runtime).await.unwrap();
    let snapshot = desktop.snapshot().await;
    let matrix_log = std::fs::read_to_string(&matrix_log).unwrap();
    let config_json =
        serde_json::to_string(&EditableConfigView::from_runtime_config(&runtime_for_view)).unwrap();

    assert_eq!(snapshot.web.kind, ServiceStatusKind::Running);
    assert_eq!(snapshot.matrix.kind, ServiceStatusKind::Running);
    assert_eq!(
        snapshot.matrix.room_id.as_deref(),
        Some("!desktop-room:matrix.fixture.invalid")
    );
    assert!(snapshot
        .web
        .url
        .as_deref()
        .unwrap_or_default()
        .starts_with("http://127.0.0.1:"));
    assert!(matrix_log.contains(r#""kind":"connect""#), "{matrix_log}");
    assert!(
        !matrix_log.contains("desktop-fixture-token"),
        "{matrix_log}"
    );
    assert!(!config_json.contains("desktop-fixture-token"));

    std::env::remove_var("ED_SENTRY_WEBUI_DIST");
    std::env::remove_var("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG");
}
