use chrono::{DateTime, Duration, Utc};
use ed_sentry::config::{LogLevelConfig, MonitorConfig};
use ed_sentry::event::{
    parse_journal_line, JournalEvent, ReceiveTextEvent, ReservoirReplenishedEvent,
};
use ed_sentry::monitor::EventMonitor;
use ed_sentry::notifier::Notification;
use std::fs;
use std::path::Path;

fn fixture_events(name: &str) -> Vec<JournalEvent> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    content
        .lines()
        .map(|line| parse_journal_line(line).unwrap())
        .collect()
}

fn run_events(
    events: &[JournalEvent],
    monitor_config: MonitorConfig,
    log_levels: LogLevelConfig,
) -> MonitorRun {
    let mut monitor = EventMonitor::new(monitor_config, log_levels);
    let mut notifications = Vec::new();
    for event in events {
        notifications.extend(monitor.process_event(event));
    }
    MonitorRun {
        monitor,
        notifications,
    }
}

fn run_events_with_monitor(
    monitor: &mut EventMonitor,
    events: &[JournalEvent],
) -> Vec<Notification> {
    let mut notifications = Vec::new();
    for event in events {
        notifications.extend(monitor.process_event(event));
    }
    notifications
}

struct MonitorRun {
    monitor: EventMonitor,
    notifications: Vec<Notification>,
}

fn notification_texts(notifications: &[Notification]) -> Vec<&str> {
    notifications
        .iter()
        .map(|notification| notification.terminal_text.as_str())
        .collect()
}

fn timestamp() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2035-01-09T10:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

fn reservoir_replenished_event(fuel_main: f64) -> JournalEvent {
    JournalEvent::ReservoirReplenished(ReservoirReplenishedEvent {
        timestamp: timestamp(),
        event: "ReservoirReplenished".to_string(),
        raw: None,
        fuel_main: Some(fuel_main),
        fuel_reservoir: Some(0.5),
    })
}

fn timed_reservoir_replenished_event(timestamp: DateTime<Utc>, fuel_main: f64) -> JournalEvent {
    JournalEvent::ReservoirReplenished(ReservoirReplenishedEvent {
        timestamp,
        event: "ReservoirReplenished".to_string(),
        raw: None,
        fuel_main: Some(fuel_main),
        fuel_reservoir: Some(0.5),
    })
}

fn res_drop_event() -> JournalEvent {
    parse_journal_line(
        r#"{"timestamp":"2035-01-09T09:59:00Z","event":"SupercruiseDestinationDrop","Type":"ResourceExtraction","Type_Localised":"Resource Extraction Site"}"#,
    )
    .unwrap()
}

fn assert_fuel_report_notification(fuel_main: f64, expected_level: u8) {
    let event = reservoir_replenished_event(fuel_main);

    let run = run_events(
        &[res_drop_event(), event],
        MonitorConfig::default(),
        LogLevelConfig::default(),
    );

    let notification = run
        .notifications
        .iter()
        .find(|notification| notification.event_type == "fuel_report")
        .unwrap();
    assert_eq!(notification.level, expected_level);
    assert!(notification.terminal_text.contains("Fuel:"));
}

#[test]
fn monitor_events_combat_notifications() {
    let events = fixture_events("journal_combat_bounty.log");

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());

    let notifications = &run.notifications;
    let texts = notification_texts(notifications);
    assert_eq!(run.monitor.state().cargo_scans, 0);
    assert_eq!(run.monitor.state().kills, 2);
    assert_eq!(run.monitor.state().bounty_total, 18_400);
    assert!(
        texts.contains(&"Scan: Viper Mk III (Competent)"),
        "{texts:?}"
    );
    assert_eq!(texts.iter().filter(|text| text.contains("Kill")).count(), 2);
    assert!(
        texts.iter().any(|text| text.contains("Cargo stolen")),
        "{texts:?}"
    );

    let scan = notifications
        .iter()
        .find(|notification| notification.event_type == "ship_scan")
        .unwrap();
    assert_eq!(scan.level, 1);
    assert!(!scan.mention);
}

#[test]
fn monitor_events_rank_promotions_default_to_level_two_mentions() {
    let events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"Promotion","Combat":4}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:01:00Z","event":"PowerplayRank","Power":"Fixture Power","Rank":3}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:02:00Z","event":"SquadronPromotion","SquadronName":"Fixture Squadron","NewRank":2}"#,
        )
        .unwrap(),
    ];

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());
    let notifications = run
        .notifications
        .iter()
        .filter(|notification| notification.event_type == "rank_promotion")
        .collect::<Vec<_>>();

    assert_eq!(notifications.len(), 3);
    assert!(notifications.iter().all(|notification| {
        notification.level == LogLevelConfig::default().rank_promotion && notification.mention
    }));
    assert!(notifications[0]
        .terminal_text
        .contains("Rank promotion: Combat 4"));
    assert!(notifications[1]
        .terminal_text
        .contains("Powerplay rank changed: Fixture Power rank 3"));
    assert!(notifications[2]
        .terminal_text
        .contains("Squadron promotion: Fixture Squadron rank 2"));
}

#[test]
fn monitor_events_output_options_change_visible_text() {
    let events = fixture_events("journal_combat_bounty.log");
    let monitor_config = MonitorConfig {
        pirate_names: false,
        bounty_faction: false,
        bounty_value: false,
        ..MonitorConfig::default()
    };

    let run = run_events(&events, monitor_config, LogLevelConfig::default());

    let notifications = &run.notifications;
    let texts = notification_texts(notifications);
    assert!(
        texts.contains(&"Scan: Viper Mk III (Competent)"),
        "{texts:?}"
    );
    assert!(texts.contains(&"Kill: Viper Mk III"), "{texts:?}");
    assert!(texts.contains(&"Kill: Bond (+5s)"), "{texts:?}");
    assert!(
        !texts.iter().any(|text| text.contains("Fixture Raider One")),
        "{texts:?}"
    );
    assert!(
        !texts.iter().any(|text| text.contains("Practice Raiders")),
        "{texts:?}"
    );
    assert!(
        !texts.iter().any(|text| text.contains("6400 cr")),
        "{texts:?}"
    );
}

#[test]
fn monitor_events_damage_notifications() {
    let events = fixture_events("journal_damage_fighter.log");

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());

    let notifications = &run.notifications;
    let texts = notification_texts(notifications);
    assert_eq!(run.monitor.state().shields_up, Some(false));
    assert_eq!(run.monitor.state().ship_hull, Some(0.0));
    assert_eq!(run.monitor.state().fighter_alive, Some(false));
    assert!(!run.monitor.state().active_session);
    assert!(
        texts.iter().any(|text| text.contains("Ship shields down!")),
        "{texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|text| text.contains("Ship shields back up")),
        "{texts:?}"
    );
    assert!(
        texts.iter().any(|text| text.contains("Fighter destroyed")),
        "{texts:?}"
    );
    assert!(
        texts.iter().any(|text| text.contains("Ship hull damaged")),
        "{texts:?}"
    );
    assert!(
        texts.iter().any(|text| text.contains("Ship destroyed")),
        "{texts:?}"
    );
    assert!(notifications
        .iter()
        .filter(|notification| notification.level >= 2)
        .all(|notification| notification.mention));
}

#[test]
fn monitor_events_mission_redirect_uses_mission_log_level_without_counting_kills() {
    let events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"MissionAccepted","MissionID":42,"Name":"Mission_Massacre_name"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:01:00Z","event":"MissionRedirected","MissionID":42,"Name":"Mission_Massacre_name"}"#,
        )
        .unwrap(),
    ];

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());

    let notifications = &run.notifications;
    let redirect = notifications
        .iter()
        .find(|notification| notification.event_type == "mission_redirected")
        .unwrap();
    assert!(redirect.terminal_text.contains("Completed kills for"));
    assert_eq!(redirect.level, LogLevelConfig::default().missions);
    assert_eq!(run.monitor.state().kills, 0);
}

#[test]
fn monitor_events_upstream_filters_fighter_events() {
    let events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"LaunchFighter","PlayerControlled":true}"#,
        )
        .unwrap(),
        parse_journal_line(r#"{"timestamp":"2035-01-09T10:01:00Z","event":"StartJump"}"#)
            .unwrap(),
        parse_journal_line(r#"{"timestamp":"2035-01-09T10:01:01Z","event":"FighterDestroyed"}"#)
            .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:02:00Z","event":"HullDamage","Fighter":true,"PlayerPilot":false,"Health":0.8}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:03:00Z","event":"HullDamage","Fighter":true,"PlayerPilot":false,"Health":0.8}"#,
        )
        .unwrap(),
    ];

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());

    let notifications = &run.notifications;
    assert!(!notifications
        .iter()
        .any(|notification| notification.event_type == "fighter_launch"));
    assert!(!notifications
        .iter()
        .any(|notification| notification.event_type == "fighter_destroyed"));
    assert_eq!(
        notifications
            .iter()
            .filter(|notification| notification.event_type == "fighter_hull")
            .count(),
        1
    );
}

#[test]
fn monitor_events_upstream_scan_counters_only_track_incoming_cargo_scans() {
    let events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Viper Mk III","PilotName":"Fixture Raider","LegalStatus":"Wanted"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:01:00Z","event":"ReceiveText","Channel":"npc","From_Localised":"Fixture Pirate","Message":"$Pirate_OnStartScanCargo"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:02:00Z","event":"ReceiveText","Channel":"npc","From_Localised":"Fixture Pirate","Message":"$Pirate_OnStartScanCargo"}"#,
        )
        .unwrap(),
    ];

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());

    assert_eq!(run.monitor.state().cargo_scans, 1);
    assert_eq!(
        run.notifications
            .iter()
            .filter(|notification| notification.event_type == "cargo_scan")
            .count(),
        1
    );
}

#[test]
fn monitor_events_upstream_session_start_backdating_and_drop_reset() {
    let mut monitor = EventMonitor::new(MonitorConfig::default(), LogLevelConfig::default());
    let events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"Bounty","Rewards":[{"Faction":"Fixture","Reward":1000}],"Target":"viper","VictimFaction":"Raiders"}"#,
        )
        .unwrap(),
    ];
    run_events_with_monitor(&mut monitor, &events);
    assert_eq!(
        monitor.state().session_started_at.unwrap(),
        DateTime::parse_from_rfc3339("2035-01-09T09:59:00Z")
            .unwrap()
            .with_timezone(&Utc)
    );

    let reset_events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:10:00Z","event":"SupercruiseDestinationDrop","Type":"$Warzone;","Type_Localised":"Conflict Zone"}"#,
        )
        .unwrap(),
    ];
    run_events_with_monitor(&mut monitor, &reset_events);
    assert_eq!(monitor.state().kills, 0);
    assert_eq!(
        monitor.state().session_started_at.unwrap(),
        DateTime::parse_from_rfc3339("2035-01-09T10:10:00Z")
            .unwrap()
            .with_timezone(&Utc)
    );
}

#[test]
fn monitor_events_upstream_missions_snapshot_and_redirect_semantics() {
    let events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T09:59:00Z","event":"SupercruiseDestinationDrop","Type":"ResourceExtraction","Type_Localised":"Resource Extraction Site"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"Missions","Active":[{"MissionID":10,"Name":"Mission_Massacre_name","Expires":100},{"MissionID":11,"Name":"Mission_CivilWar_name","Expires":100}]}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:01:00Z","event":"Missions","Active":[{"MissionID":12,"Name":"Mission_Massacre_name","Expires":100}]}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:02:00Z","event":"MissionRedirected","MissionID":99,"Name":"Mission_Massacre_name"}"#,
        )
        .unwrap(),
    ];

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());
    let notifications = &run.notifications;
    let mission_loads = notifications
        .iter()
        .filter(|notification| notification.event_type == "missions_snapshot")
        .count();
    let redirect = notifications
        .iter()
        .find(|notification| notification.event_type == "mission_redirected")
        .unwrap();

    assert_eq!(mission_loads, 1);
    assert_eq!(
        redirect.terminal_text,
        "Completed kills for all missions! (1/1)"
    );
    assert!(run
        .monitor
        .render_dynamic_title(
            DateTime::parse_from_rfc3339("2035-01-09T10:02:00Z")
                .unwrap()
                .with_timezone(&Utc)
        )
        .contains("🎯 1/1"));
}

#[test]
fn monitor_events_upstream_summary_scans_faction_and_merits() {
    let log_levels = LogLevelConfig {
        summary_scans: 1,
        summary_faction: 1,
        merits: 1,
        summary_merits: 1,
        ..LogLevelConfig::default()
    };
    let events = [
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:00:00Z","event":"ReceiveText","Channel":"npc","From_Localised":"Pirate One","Message":"$Pirate_OnStartScanCargo"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:02:00Z","event":"ReceiveText","Channel":"npc","From_Localised":"Pirate Two","Message":"$Pirate_OnStartScanCargo"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:03:00Z","event":"Bounty","Rewards":[{"Faction":"Fixture","Reward":1000}],"Target":"viper","VictimFaction":"Raiders"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:03:05Z","event":"PowerplayMerits","MeritsGained":12,"Power":"Aisling Duval"}"#,
        )
        .unwrap(),
        parse_journal_line(
            r#"{"timestamp":"2035-01-09T10:04:00Z","event":"Bounty","Rewards":[{"Faction":"Fixture","Reward":3000}],"Target":"viper","VictimFaction":"Raiders"}"#,
        )
        .unwrap(),
    ];

    let mut run = run_events(&events, MonitorConfig::default(), log_levels);
    run.notifications
        .extend(run.monitor.finish("Journal.test.log", timestamp()));
    let output = run
        .notifications
        .iter()
        .map(|notification| notification.terminal_text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(output.contains("Merits: +12 (Aisling Duval)"), "{output}");
    assert!(
        output.contains("-> Faction: 2 (60.0/h | 100%) [Raiders]"),
        "{output}"
    );
    assert!(output.contains("-> Scans: 2 (30.0/h | 2m)"), "{output}");
    assert!(
        output.contains("-> Merits: 12 (720/h | 6/kill)"),
        "{output}"
    );
}

#[test]
fn monitor_events_non_massacre_redirect_does_not_emit_mission_progress() {
    let events = fixture_events("journal_missions.log");

    let run = run_events(&events, MonitorConfig::default(), LogLevelConfig::default());

    assert_eq!(run.monitor.state().mission_completed, 0);
    assert!(!run
        .notifications
        .iter()
        .any(|notification| notification.event_type == "mission_redirected"));
}

#[test]
fn monitor_events_receive_text_pirate_scan_updates_state_and_notifies() {
    let event = JournalEvent::ReceiveText(ReceiveTextEvent {
        timestamp: timestamp(),
        event: "ReceiveText".to_string(),
        raw: None,
        from: Some("npc_fixture_pirate".to_string()),
        from_localised: Some("Fixture Pirate".to_string()),
        message: Some("$Pirate_OnStartScanCargo".to_string()),
        channel: Some("npc".to_string()),
    });

    let run = run_events(
        &[event],
        MonitorConfig::default(),
        LogLevelConfig::default(),
    );

    let notifications = &run.notifications;
    assert_eq!(run.monitor.state().cargo_scans, 1);
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].event_type, "cargo_scan");
    assert!(notifications[0].terminal_text.contains("Cargo scan"));
    assert_eq!(
        notifications[0].level,
        LogLevelConfig::default().scan_incoming
    );
}

#[test]
fn monitor_events_reservoir_replenished_reports_normal_fuel_level() {
    assert_fuel_report_notification(32.0, LogLevelConfig::default().fuel_report);
}

#[test]
fn monitor_events_reservoir_replenished_reports_fuel_eta_after_previous_sample() {
    let start = timestamp();
    let run = run_events(
        &[
            res_drop_event(),
            timed_reservoir_replenished_event(start, 63.0),
            timed_reservoir_replenished_event(start + Duration::minutes(10), 62.5),
        ],
        MonitorConfig::default(),
        LogLevelConfig::default(),
    );

    let notification = run
        .notifications
        .iter()
        .rfind(|notification| notification.event_type == "fuel_report")
        .unwrap();
    assert_eq!(notification.terminal_text, "Fuel: 98% remaining (~20h50m)");
}

#[test]
fn monitor_events_reservoir_replenished_reports_low_fuel_level() {
    assert_fuel_report_notification(8.0, LogLevelConfig::default().fuel_low);
}

#[test]
fn monitor_events_reservoir_replenished_reports_critical_fuel_level() {
    assert_fuel_report_notification(1.0, LogLevelConfig::default().fuel_critical);
}

#[test]
fn monitor_events_level_zero_updates_state_without_delivery() {
    let events = fixture_events("journal_combat_bounty.log");
    let log_levels = LogLevelConfig {
        scan_hard: 0,
        scan_easy: 0,
        kill_easy: 0,
        kill_hard: 0,
        cargo_lost: 0,
        fuel_report: 0,
        ..LogLevelConfig::default()
    };

    let run = run_events(&events, MonitorConfig::default(), log_levels);

    assert_eq!(run.monitor.state().cargo_scans, 0);
    assert_eq!(run.monitor.state().kills, 2);
    assert_eq!(run.monitor.state().bounty_total, 18_400);
    assert!(run
        .notifications
        .iter()
        .all(|notification| notification.level == 0));
}
