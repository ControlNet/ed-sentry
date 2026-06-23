use chrono::{TimeZone, Utc};
use ed_sentry::app::runtime::{ConfiguredJournalSelector, MonitorRuntime};
use ed_sentry::app::{
    JournalSourceView, MatrixStartupStatus, MissionProgressView, WebStartupStatus,
};
use ed_sentry::config::{AppConfig, CliConfigOverrides};

#[test]
fn runtime_service_emits_sanitized_snapshot_and_notifications_from_fixture() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journal_path = temp_dir.path().join("Journal.2035-01-03T100000.01.log");
    std::fs::write(
        &journal_path,
        concat!(
            r#"{"timestamp":"2035-01-03T10:00:00Z","event":"Commander","Name":"Cmdr Fixture\nInjected\u001b[2K"}"#,
            "\n",
            r#"{"timestamp":"2035-01-03T10:01:00Z","event":"Location","StarSystem":"Runtime\nSystem\u001b[2K","Docked":false}"#,
            "\n",
            r#"{"timestamp":"2035-01-03T10:02:00Z","event":"MissionAccepted","Faction":"Runtime\nGuild\u001b[2K","Name":"Mission_Massacre","LocalisedName":"Clean\nmassacre\u001b[2K","MissionID":7002001,"DestinationSystem":"Target\nRuntime\u001b[2K","DestinationStation":"Runtime\nStation\u001b[2K","TargetFaction":"Practice\nRaiders\u001b[2K","Target":"Pirate\nWing\u001b[2K","TargetType":"Pirate","KillCount":2,"Reward":50000}"#,
            "\n",
            r#"{"timestamp":"2035-01-03T10:03:00Z","event":"ShipTargeted","TargetLocked":true,"ScanStage":3,"Ship":"viper","Ship_Localised":"Runtime\nViper\u001b[2K","PilotName":"Runtime Raider","LegalStatus":"Wanted"}"#,
            "\n"
        ),
    )
    .unwrap();
    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        set_file: Some(journal_path.clone()),
        ..CliConfigOverrides::default()
    });
    let started_at = Utc.with_ymd_and_hms(2035, 1, 3, 9, 59, 0).unwrap();
    let now = Utc.with_ymd_and_hms(2035, 1, 3, 10, 4, 0).unwrap();

    let mut runtime = MonitorRuntime::start(
        &config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap();
    let batch = runtime.process_preload(started_at);
    let snapshot = runtime.snapshot(now);
    let configured_source = JournalSourceView::from_runtime_config(&config);
    let json = serde_json::to_string(&snapshot).unwrap();
    let full_path = journal_path.to_string_lossy();

    assert_eq!(
        runtime.startup().commander.as_deref(),
        Some("Cmdr Fixture Injected [2K")
    );
    assert!(batch
        .notifications
        .iter()
        .any(|item| item.notification.event_type == "ship_scan"));
    assert_eq!(
        snapshot.session.commander.as_deref(),
        Some("Cmdr Fixture Injected [2K")
    );
    assert_eq!(
        snapshot.session.system.as_deref(),
        Some("Runtime System [2K")
    );
    assert_eq!(snapshot.missions.active_count, 1);
    let mission = &snapshot.missions.items[0];
    assert_eq!(snapshot.notifications.len(), 1);
    assert!(snapshot.notifications[0].text.contains("Runtime Viper [2K"));
    assert_eq!(snapshot.event_feed.len(), 1);

    let mut violations = Vec::new();
    if snapshot.journal_source.selected_file.as_deref() != Some("Journal.2035-01-03T100000.01.log")
    {
        violations.push(format!(
            "snapshot selected_file leaked or changed unexpectedly: {:?}",
            snapshot.journal_source.selected_file
        ));
    }
    if configured_source.selected_file.as_deref() != Some("Journal.2035-01-03T100000.01.log") {
        violations.push(format!(
            "config selected_file leaked or changed unexpectedly: {:?}",
            configured_source.selected_file
        ));
    }
    if json.contains(full_path.as_ref()) {
        violations.push(format!(
            "snapshot JSON contains full Journal path: {full_path}"
        ));
    }
    if mission.display_name != "Clean massacre [2K" {
        violations.push(format!(
            "mission display_name was not line-safe: {:?}",
            mission.display_name
        ));
    }
    if mission.issuing_faction.as_deref() != Some("Runtime Guild [2K") {
        violations.push(format!(
            "mission issuing_faction was not line-safe: {:?}",
            mission.issuing_faction
        ));
    }
    if mission.target_faction.as_deref() != Some("Practice Raiders [2K") {
        violations.push(format!(
            "mission target_faction was not line-safe: {:?}",
            mission.target_faction
        ));
    }
    if mission.destination_system.as_deref() != Some("Target Runtime [2K") {
        violations.push(format!(
            "mission destination_system was not line-safe: {:?}",
            mission.destination_system
        ));
    }
    if mission.destination_station.as_deref() != Some("Runtime Station [2K") {
        violations.push(format!(
            "mission destination_station was not line-safe: {:?}",
            mission.destination_station
        ));
    }
    match &mission.progress {
        ed_sentry::app::MissionProgressView::Massacre { target, .. } => {
            if target.as_deref() != Some("Pirate Wing [2K") {
                violations.push(format!(
                    "mission progress target was not line-safe: {target:?}"
                ));
            }
        }
        other => panic!("expected massacre progress, got {other:?}"),
    }
    if json.contains("\\n") {
        violations.push("snapshot JSON contains escaped newline text".to_string());
    }
    if json.contains("\\u001b") {
        violations.push("snapshot JSON contains escaped ANSI/control text".to_string());
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}

#[test]
fn runtime_service_rebuilds_mission_details_from_recent_journal_history() {
    let temp_dir = tempfile::tempdir().unwrap();
    let accepted_path = temp_dir.path().join("Journal.2035-01-04T120000.01.log");
    let active_path = temp_dir.path().join("Journal.2035-01-04T121000.01.log");
    std::fs::write(
        &accepted_path,
        concat!(
            r#"{"timestamp":"2035-01-04T12:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
            "\n",
            r#"{"timestamp":"2035-01-04T12:01:00Z","event":"Location","StarSystem":"Mission Test System","SystemAddress":100,"StationName":"Task Board Hub","MarketID":200,"Docked":true}"#,
            "\n",
            r#"{"timestamp":"2035-01-04T12:02:00Z","event":"MissionAccepted","Faction":"Issuer A","Name":"Mission_Massacre_name","MissionID":7001003,"DestinationSystem":"Target System","TargetFaction":"Fixture Raiders","Target":"Pirate Wing","TargetType":"MissionUtil_FactionTag_Pirate","KillCount":12,"Reward":500000}"#,
            "\n",
        ),
    )
    .unwrap();
    std::fs::write(
        &active_path,
        r#"{"timestamp":"2035-01-04T12:10:00Z","event":"Missions","Active":[{"MissionID":7001003,"Name":"Mission_Massacre_name","PassengerMission":false,"Expires":1800}],"Failed":[],"Complete":[]}"#,
    )
    .unwrap();

    let config = AppConfig::default().into_runtime(&CliConfigOverrides {
        journal_folder: Some(temp_dir.path().to_path_buf()),
        ..CliConfigOverrides::default()
    });
    let mut runtime = MonitorRuntime::start(
        &config,
        &mut ConfiguredJournalSelector,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
    .unwrap();
    runtime.process_preload(Utc.with_ymd_and_hms(2035, 1, 4, 12, 11, 0).unwrap());
    let snapshot = runtime.snapshot(Utc.with_ymd_and_hms(2035, 1, 4, 12, 11, 0).unwrap());
    let mission = snapshot
        .missions
        .items
        .iter()
        .find(|mission| mission.mission_id == 7001003)
        .unwrap();

    assert_eq!(mission.kind, "massacre");
    assert_eq!(mission.target_faction.as_deref(), Some("Fixture Raiders"));
    assert_eq!(mission.destination_system.as_deref(), Some("Target System"));
    match &mission.progress {
        MissionProgressView::Massacre {
            kills,
            kill_count,
            display,
            ..
        } => {
            assert_eq!((*kills, *kill_count), (0, 12));
            assert_eq!(display, "0/12 kills");
        }
        other => panic!("expected massacre progress, got {other:?}"),
    }
}
