use ed_sentry::event::parse_journal_line;
use ed_sentry::mission::{MissionKind, MissionProgress, MissionState, MissionTracker};

fn apply_lines(lines: &[&str]) -> MissionTracker {
    let mut tracker = MissionTracker::new();
    for line in lines {
        let event = parse_journal_line(line).unwrap();
        tracker.apply_event(&event);
    }
    tracker
}

#[test]
fn mission_tracker_classifies_massacre_missions_and_counts_bounty_progress() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:00:00Z","event":"LoadGame","Commander":"Cmdr Fixture","Odyssey":true}"#,
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"Location","StarSystem":"Origin System","SystemAddress":123,"StationName":"Origin Hub","MarketID":456,"Docked":true}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"MissionAccepted","Faction":"Issuer A","Name":"Mission_Massacre_name","MissionID":7001,"TargetFaction":"Fixture Raiders","TargetType":"MissionUtil_FactionTag_Pirate","DestinationSystem":"Target System","KillCount":2,"Reward":100000,"Wing":true}"#,
        r#"{"timestamp":"2035-01-04T12:03:00Z","event":"Bounty","TotalReward":1000,"VictimFaction":"Fixture Raiders","Target":"viper"}"#,
        r#"{"timestamp":"2035-01-04T12:04:00Z","event":"MissionRedirected","MissionID":7001,"Name":"Mission_Massacre_name","NewDestinationSystem":"Origin System","NewDestinationStation":"Origin Hub"}"#,
    ]);

    let mission = tracker.mission(7001).unwrap();
    assert_eq!(mission.kind, MissionKind::Massacre);
    assert_eq!(mission.state, MissionState::Redirected);
    assert_eq!(mission.origin.system_name.as_deref(), Some("Origin System"));
    assert_eq!(mission.origin.station_name.as_deref(), Some("Origin Hub"));
    assert_eq!(mission.origin.market_id, Some(456));
    assert_eq!(mission.origin.odyssey, Some(true));
    assert_eq!(mission.destination_station.as_deref(), Some("Origin Hub"));
    assert_eq!(mission.reward, Some(100000));
    assert_eq!(mission.wing, Some(true));
    assert_eq!(
        mission.progress,
        MissionProgress::Massacre {
            target: None,
            target_type: Some("MissionUtil_FactionTag_Pirate".to_string()),
            target_faction: Some("Fixture Raiders".to_string()),
            target_system: Some("Target System".to_string()),
            kill_count: 2,
            kills: 2,
        }
    );
}

#[test]
fn mission_tracker_classifies_trade_missions_and_updates_cargo_depot_progress() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"Location","StarSystem":"Origin System","SystemAddress":123,"StationName":"Origin Hub","MarketID":456,"Docked":true}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"MissionAccepted","Faction":"Issuer B","Name":"Mission_Delivery_name","MissionID":8001,"DestinationSystem":"Delivery System","DestinationStation":"Delivery Port","Expiry":"2035-01-05T12:03:00Z","Influence":"Med","Reputation":"Med","Reward":50000,"Commodity":"syntheticfabric","Commodity_Localised":"Synthetic Fabric","Count":12}"#,
        r#"{"timestamp":"2035-01-04T12:03:00Z","event":"CargoDepot","MissionID":8001,"CargoType":"syntheticfabric","CargoType_Localised":"Synthetic Fabric","ItemsCollected":12,"ItemsDelivered":5,"TotalItemsToDeliver":12,"Progress":0.42}"#,
        r#"{"timestamp":"2035-01-04T12:04:00Z","event":"MissionCompleted","MissionID":8001,"Name":"Mission_Delivery_name","Reward":50000}"#,
    ]);

    let mission = tracker.mission(8001).unwrap();
    assert_eq!(mission.kind, MissionKind::Trade);
    assert_eq!(mission.state, MissionState::Completed);
    assert_eq!(mission.issuing_faction.as_deref(), Some("Issuer B"));
    assert_eq!(
        mission.destination_system.as_deref(),
        Some("Delivery System")
    );
    assert_eq!(
        mission.destination_station.as_deref(),
        Some("Delivery Port")
    );
    assert_eq!(mission.influence.as_deref(), Some("Med"));
    assert_eq!(mission.reputation.as_deref(), Some("Med"));
    assert!(mission.completion_time.is_some());
    assert_eq!(
        mission.progress,
        MissionProgress::Trade {
            commodity: Some("syntheticfabric".to_string()),
            commodity_localised: Some("Synthetic Fabric".to_string()),
            count: 12,
            items_collected: 12,
            items_delivered: 5,
        }
    );
}

#[test]
fn mission_tracker_missions_snapshot_removes_missing_active_missions() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"MissionAccepted","Name":"Mission_Delivery_name","MissionID":1,"Count":1}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"MissionAccepted","Name":"Mission_Delivery_name","MissionID":2,"Count":1}"#,
        r#"{"timestamp":"2035-01-04T12:03:00Z","event":"Missions","Active":[{"MissionID":2,"Name":"Mission_Delivery_name","PassengerMission":false,"Expires":1800}],"Failed":[],"Complete":[]}"#,
    ]);

    assert!(tracker.mission(1).is_none());
    assert!(tracker.mission(2).is_some());
}

#[test]
fn mission_tracker_empty_active_snapshot_clears_active_missions() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"MissionAccepted","Name":"Mission_Delivery_name","MissionID":1,"Count":1}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"Missions","Active":[],"Failed":[],"Complete":[]}"#,
    ]);

    assert!(tracker.mission(1).is_none());
}

#[test]
fn mission_tracker_creates_active_missions_from_snapshot_without_accept_event() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"Location","StarSystem":"Mission Test System","SystemAddress":100,"StationName":"Task Board Hub","MarketID":200,"Docked":true}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"Missions","Active":[{"MissionID":7001001,"Name":"Mission_Delivery_name","PassengerMission":false,"Expires":1800}],"Failed":[],"Complete":[]}"#,
    ]);

    let mission = tracker.mission(7001001).unwrap();
    assert_eq!(mission.state, MissionState::Active);
    assert_eq!(mission.kind, MissionKind::Trade);
    assert_eq!(mission.name.as_deref(), Some("Mission_Delivery_name"));
    assert_eq!(
        mission.expiry.map(|expiry| expiry.to_rfc3339()),
        Some("2035-01-04T12:32:00+00:00".to_string())
    );
    assert_eq!(
        mission.origin.system_name.as_deref(),
        Some("Mission Test System")
    );
    assert_eq!(
        mission.origin.station_name.as_deref(),
        Some("Task Board Hub")
    );
}

#[test]
fn mission_tracker_massacre_bounty_progresses_one_mission_per_issuing_faction() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"MissionAccepted","Faction":"Issuer A","Name":"Mission_Massacre_name","MissionID":1,"TargetFaction":"Raiders","KillCount":3}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"MissionAccepted","Faction":"Issuer A","Name":"Mission_Massacre_name","MissionID":2,"TargetFaction":"Raiders","KillCount":3}"#,
        r#"{"timestamp":"2035-01-04T12:03:00Z","event":"MissionAccepted","Faction":"Issuer B","Name":"Mission_Massacre_name","MissionID":3,"TargetFaction":"Raiders","KillCount":3}"#,
        r#"{"timestamp":"2035-01-04T12:04:00Z","event":"Bounty","TotalReward":1000,"VictimFaction":"Raiders","Target":"viper"}"#,
    ]);

    assert_eq!(massacre_kills(&tracker, 1), 1);
    assert_eq!(massacre_kills(&tracker, 2), 0);
    assert_eq!(massacre_kills(&tracker, 3), 1);
}

#[test]
fn mission_tracker_ignores_invalid_massacre_bounties() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"MissionAccepted","Faction":"Issuer A","Name":"Mission_Massacre_name","MissionID":1,"TargetFaction":"Raiders","KillCount":3}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"Bounty","TotalReward":0,"VictimFaction":"Raiders","Target":"viper"}"#,
        r#"{"timestamp":"2035-01-04T12:03:00Z","event":"Bounty","TotalReward":1000,"VictimFaction":"Raiders","Target":"suit_enemy"}"#,
        r#"{"timestamp":"2035-01-04T12:04:00Z","event":"Bounty","TotalReward":1000,"VictimFaction":"faction_none","Target":"viper"}"#,
    ]);

    assert_eq!(massacre_kills(&tracker, 1), 0);
}

#[test]
fn mission_tracker_completion_donation_and_failed_state_update_rewards() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"MissionAccepted","Name":"Mission_Delivery_name","MissionID":1,"Reward":50000,"Count":1}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"MissionCompleted","MissionID":1,"Reward":0,"Donated":12000}"#,
        r#"{"timestamp":"2035-01-04T12:03:00Z","event":"MissionAccepted","Name":"Mission_Delivery_name","MissionID":2,"Reward":50000,"Count":1}"#,
        r#"{"timestamp":"2035-01-04T12:04:00Z","event":"MissionFailed","MissionID":2,"Fine":1000}"#,
    ]);

    assert_eq!(tracker.mission(1).unwrap().reward, Some(-12000));
    assert_eq!(tracker.mission(2).unwrap().reward, Some(0));
    assert_eq!(tracker.mission(2).unwrap().state, MissionState::Failed);
}

#[test]
fn mission_tracker_uses_docked_and_undocked_origin_context() {
    let tracker = apply_lines(&[
        r#"{"timestamp":"2035-01-04T12:01:00Z","event":"Docked","StarSystem":"Dock System","SystemAddress":44,"StationName":"Dock Hub","MarketID":55}"#,
        r#"{"timestamp":"2035-01-04T12:02:00Z","event":"MissionAccepted","Name":"Mission_Delivery_name","MissionID":1,"Count":1}"#,
        r#"{"timestamp":"2035-01-04T12:03:00Z","event":"Undocked"}"#,
        r#"{"timestamp":"2035-01-04T12:04:00Z","event":"MissionAccepted","Name":"Mission_Delivery_name","MissionID":2,"Count":1}"#,
    ]);

    let docked = tracker.mission(1).unwrap();
    assert_eq!(docked.origin.system_name.as_deref(), Some("Dock System"));
    assert_eq!(docked.origin.station_name.as_deref(), Some("Dock Hub"));
    assert_eq!(docked.origin.market_id, Some(55));

    let undocked = tracker.mission(2).unwrap();
    assert_eq!(undocked.origin.system_name.as_deref(), Some("Dock System"));
    assert_eq!(undocked.origin.station_name, None);
    assert_eq!(undocked.origin.market_id, None);
}

fn massacre_kills(tracker: &MissionTracker, mission_id: u64) -> u64 {
    match &tracker.mission(mission_id).unwrap().progress {
        MissionProgress::Massacre { kills, .. } => *kills,
        other => panic!("expected massacre progress, got {other:?}"),
    }
}
