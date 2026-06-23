use chrono::{DateTime, Duration, TimeZone, Utc};
use ed_sentry::event::{parse_journal_line, LoadoutEvent};
use ed_sentry::event::{
    BasicJournalEvent, BountyEvent, BountyReward, CommanderEvent, FactionKillBondEvent,
    HullDamageEvent, JournalEvent, LaunchFighterEvent, LoadGameEvent, LocationEvent, MissionEvent,
    MusicEvent, ShieldStateEvent, ShipTargetedEvent, SupercruiseDestinationDropEvent, TravelEvent,
};
use ed_sentry::state::SessionState;
use std::fs;
use std::path::Path;

fn timestamp(minutes: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2035, 2, 10, 12, 0, 0)
        .single()
        .unwrap()
        + Duration::minutes(minutes)
}

fn basic(event: &str, minutes: i64) -> BasicJournalEvent {
    BasicJournalEvent {
        timestamp: timestamp(minutes),
        event: event.to_string(),
        raw: None,
    }
}

fn location(minutes: i64, body_type: Option<&str>, body: Option<&str>) -> JournalEvent {
    JournalEvent::Location(LocationEvent {
        timestamp: timestamp(minutes),
        event: "Location".to_string(),
        raw: None,
        star_system: Some("Fixture Ring System".to_string()),
        system_address: Some(100),
        body: body.map(str::to_string),
        body_type: body_type.map(str::to_string),
        docked: Some(false),
        station_name: None,
        station_name_localised: None,
        market_id: None,
    })
}

fn res_drop(minutes: i64, destination_type: &str) -> JournalEvent {
    JournalEvent::SupercruiseDestinationDrop(SupercruiseDestinationDropEvent {
        timestamp: timestamp(minutes),
        event: "SupercruiseDestinationDrop".to_string(),
        raw: None,
        destination_type: Some(destination_type.to_string()),
        destination_type_localised: Some(destination_type.to_string()),
    })
}

fn travel(event: &str, minutes: i64) -> TravelEvent {
    TravelEvent {
        timestamp: timestamp(minutes),
        event: event.to_string(),
        raw: None,
        star_system: Some("Fixture System".to_string()),
        system_address: Some(200),
    }
}

fn incomplete_targeted(
    minutes: i64,
    legal_status: Option<&str>,
    pilot_name: Option<&str>,
) -> JournalEvent {
    JournalEvent::ShipTargeted(ShipTargetedEvent {
        timestamp: timestamp(minutes),
        event: "ShipTargeted".to_string(),
        raw: None,
        target_locked: Some(true),
        scan_stage: Some(0),
        ship: Some("hauler".to_string()),
        ship_localised: Some("Hauler".to_string()),
        pilot_name: pilot_name.map(str::to_string),
        pilot_name_localised: pilot_name.map(str::to_string),
        pilot_rank: Some("Mostly Harmless".to_string()),
        legal_status: legal_status.map(str::to_string),
    })
}

fn bounty(minutes: i64, total_reward: Option<u64>, reward_items: &[u64]) -> JournalEvent {
    JournalEvent::Bounty(BountyEvent {
        timestamp: timestamp(minutes),
        event: "Bounty".to_string(),
        raw: None,
        total_reward,
        rewards: Some(
            reward_items
                .iter()
                .map(|reward| BountyReward {
                    faction: Some("Fixture Security".to_string()),
                    reward: Some(*reward),
                })
                .collect(),
        ),
        victim_faction: Some("Fixture Raiders".to_string()),
        victim_faction_localised: None,
        target: Some("viper".to_string()),
        target_localised: Some("Viper Mk III".to_string()),
        pilot_name_localised: None,
    })
}

fn kill_bond(minutes: i64, reward: u64) -> JournalEvent {
    JournalEvent::FactionKillBond(FactionKillBondEvent {
        timestamp: timestamp(minutes),
        event: "FactionKillBond".to_string(),
        raw: None,
        reward: Some(reward),
        awarding_faction: Some("Fixture Navy".to_string()),
        victim_faction: Some("Fixture Raiders".to_string()),
        victim_faction_localised: None,
    })
}

fn mission(event: &str, minutes: i64, mission_id: u64, name: &str) -> MissionEvent {
    MissionEvent {
        timestamp: timestamp(minutes),
        event: event.to_string(),
        raw: None,
        mission_id: Some(mission_id),
        name: Some(name.to_string()),
        localised_name: None,
        faction: None,
        target_faction: None,
        target: None,
        target_type: None,
        destination_system: None,
        destination_station: None,
        destination_settlement: None,
        new_destination_system: None,
        new_destination_station: None,
        old_destination_system: None,
        old_destination_station: None,
        expiry: None,
        influence: None,
        reputation: None,
        reward: None,
        donated: None,
        fine: None,
        wing: None,
        commodity: None,
        commodity_localised: None,
        count: None,
        kill_count: None,
    }
}

#[test]
fn session_state_tracks_identity_status_and_damage_from_typed_events() {
    let mut state = SessionState::new();

    state.apply_event(&JournalEvent::Commander(CommanderEvent {
        timestamp: timestamp(0),
        event: "Commander".to_string(),
        raw: None,
        name: Some("Cmdr Fixture State".to_string()),
    }));
    state.apply_event(&JournalEvent::LoadGame(LoadGameEvent {
        timestamp: timestamp(1),
        event: "LoadGame".to_string(),
        raw: None,
        commander: Some("Cmdr Fixture State".to_string()),
        ship: Some("krait_mkii".to_string()),
        ship_localised: Some("Krait Mk II".to_string()),
        game_mode: Some("Open".to_string()),
        odyssey: Some(true),
    }));
    state.apply_event(&JournalEvent::Loadout(LoadoutEvent {
        timestamp: timestamp(2),
        event: "Loadout".to_string(),
        raw: None,
        ship: Some("python".to_string()),
        ship_localised: Some("Python".to_string()),
        fuel_capacity_main: Some(32.0),
    }));
    state.apply_event(&location(3, None, Some("Fixture Belt")));
    state.apply_event(&JournalEvent::ShieldState(ShieldStateEvent {
        timestamp: timestamp(4),
        event: "ShieldState".to_string(),
        raw: None,
        shields_up: Some(false),
    }));
    state.apply_event(&JournalEvent::HullDamage(HullDamageEvent {
        timestamp: timestamp(5),
        event: "HullDamage".to_string(),
        raw: None,
        health: Some(0.72),
        player_pilot: Some(true),
        fighter: Some(false),
    }));
    state.apply_event(&JournalEvent::LaunchFighter(LaunchFighterEvent {
        timestamp: timestamp(6),
        event: "LaunchFighter".to_string(),
        raw: None,
        player_controlled: Some(false),
    }));
    state.apply_event(&JournalEvent::HullDamage(HullDamageEvent {
        timestamp: timestamp(7),
        event: "HullDamage".to_string(),
        raw: None,
        health: Some(0.44),
        player_pilot: Some(false),
        fighter: Some(true),
    }));
    state.apply_event(&JournalEvent::FighterDestroyed(basic(
        "FighterDestroyed",
        8,
    )));

    assert_eq!(state.commander.as_deref(), Some("Cmdr Fixture State"));
    assert_eq!(state.ship.as_deref(), Some("Python"));
    assert_eq!(state.system.as_deref(), Some("Fixture Ring System"));
    assert_eq!(state.mode.as_deref(), Some("Open"));
    assert_eq!(state.shields_up, Some(false));
    assert_eq!(state.ship_hull, Some(0.72));
    assert_eq!(state.fighter_hull, Some(0.0));
    assert_eq!(state.fighter_alive, Some(false));
}

#[test]
fn session_state_preserves_localised_ship_display_when_loadout_only_has_raw_key() {
    let mut state = SessionState::new();

    state.apply_event(&JournalEvent::LoadGame(LoadGameEvent {
        timestamp: timestamp(0),
        event: "LoadGame".to_string(),
        raw: None,
        commander: Some("Cmdr Fixture State".to_string()),
        ship: Some("Type9_Military".to_string()),
        ship_localised: Some("Type-10 Defender".to_string()),
        game_mode: Some("Open".to_string()),
        odyssey: Some(true),
    }));
    state.apply_event(&JournalEvent::Loadout(LoadoutEvent {
        timestamp: timestamp(1),
        event: "Loadout".to_string(),
        raw: None,
        ship: Some("type9_military".to_string()),
        ship_localised: None,
        fuel_capacity_main: Some(64.0),
    }));

    assert_eq!(state.ship.as_deref(), Some("Type-10 Defender"));
}

#[test]
fn session_state_bounty_and_faction_kill_bond_increment_observed_kills() {
    let mut state = SessionState::new();

    state.apply_event(&bounty(0, Some(6_400), &[3_200, 3_200]));
    state.apply_event(&kill_bond(5, 12_000));

    assert!(state.active_session);
    assert_eq!(state.session_started_at, Some(timestamp(-1)));
    assert_eq!(state.kills, 2);
    assert_eq!(state.bounty_total, 15_200);
    assert_eq!(state.last_kill_at, Some(timestamp(5)));
    assert_eq!(state.kill_timestamps(), &[timestamp(0), timestamp(5)]);
    assert!((state.total_kill_rate_per_hour_at(timestamp(60)) - 120.0 / 61.0).abs() < 1e-12);
    assert_eq!(state.recent_kill_rate_per_hour_at(timestamp(10)), 12.0);
}

#[test]
fn session_state_bounty_uses_reward_items_when_total_reward_is_absent() {
    let mut state = SessionState::new();

    state.apply_event(&bounty(0, None, &[1_000, 2_500]));

    assert_eq!(state.kills, 1);
    assert_eq!(state.bounty_total, 1_000);
}

#[test]
fn session_state_missions_not_kills() {
    let mut state = SessionState::new();

    state.apply_event(&JournalEvent::MissionAccepted(mission(
        "MissionAccepted",
        0,
        42,
        "Mission_Massacre_name",
    )));
    state.apply_event(&JournalEvent::MissionRedirected(mission(
        "MissionRedirected",
        1,
        42,
        "Mission_Massacre_name",
    )));

    assert_eq!(state.mission_total, 1);
    assert_eq!(state.mission_completed, 1);
    assert_eq!(state.kills, 0);
    assert_eq!(state.bounty_total, 0);
    assert!(state.active_massacre_mission_ids.contains(&42));

    state.apply_event(&JournalEvent::MissionCompleted(mission(
        "MissionCompleted",
        2,
        42,
        "Mission_Massacre_name",
    )));

    assert!(!state.active_massacre_mission_ids.contains(&42));
}

#[test]
fn session_state_massacre_matching_uses_localised_name_and_case_insensitive_text() {
    let mut state = SessionState::new();
    let mut localised_massacre = mission("MissionAccepted", 0, 90, "Mission_Delivery_name");
    localised_massacre.localised_name = Some("Wing MASSACRE assignment".to_string());

    state.apply_event(&JournalEvent::MissionAccepted(localised_massacre));
    state.apply_event(&JournalEvent::MissionAccepted(mission(
        "MissionAccepted",
        1,
        91,
        "Mission_Delivery_name",
    )));
    state.apply_event(&JournalEvent::MissionFailed(mission(
        "MissionFailed",
        2,
        90,
        "Mission_Delivery_name",
    )));

    assert_eq!(state.mission_total, 0);
    assert_eq!(state.mission_completed, 0);
    assert!(state.active_massacre_mission_ids.is_empty());
}

#[test]
fn session_state_start_end_rules() {
    let end_events = [
        JournalEvent::SupercruiseEntry(travel("SupercruiseEntry", 1)),
        JournalEvent::FSDJump(travel("FSDJump", 1)),
        JournalEvent::Music(MusicEvent {
            timestamp: timestamp(1),
            event: "Music".to_string(),
            raw: None,
            music_track: Some("MainMenu".to_string()),
        }),
        JournalEvent::Shutdown(basic("Shutdown", 1)),
        JournalEvent::Died(basic("Died", 1)),
    ];

    for (start, expected_start) in [
        (res_drop(0, "ResourceExtraction"), timestamp(0)),
        (
            location(0, Some("PlanetaryRing"), Some("Fixture Ring")),
            timestamp(0),
        ),
        (bounty(0, Some(1), &[1]), timestamp(-1)),
        (kill_bond(0, 1), timestamp(-1)),
    ] {
        for end in &end_events {
            let mut state = SessionState::new();
            state.apply_event(&start);
            assert!(state.active_session, "{start:?} should start a session");
            assert_eq!(state.session_started_at, Some(expected_start));

            state.apply_event(end);

            assert!(!state.active_session, "{end:?} should end a session");
            assert_eq!(state.session_ended_at, Some(timestamp(1)));
        }
    }
}

#[test]
fn session_state_non_afk_boundaries_do_not_start_or_end_session() {
    let mut state = SessionState::new();

    state.apply_event(&res_drop(0, "Station"));
    state.apply_event(&location(1, None, Some("Fixture Station")));
    state.apply_event(&incomplete_targeted(
        2,
        Some("Clean"),
        Some("Fixture Trader"),
    ));

    assert!(!state.active_session);
    assert_eq!(state.cargo_scans, 0);

    state.apply_event(&bounty(3, Some(10), &[10]));
    state.apply_event(&JournalEvent::Music(MusicEvent {
        timestamp: timestamp(4),
        event: "Music".to_string(),
        raw: None,
        music_track: Some("Combat".to_string()),
    }));

    assert!(state.active_session);
    assert_eq!(state.session_ended_at, None);
}

#[test]
fn session_state_scan_timestamps_and_rates_update_from_incoming_cargo_scans() {
    let mut state = SessionState::new();

    state.record_incoming_scan(timestamp(0));
    state.record_incoming_scan(timestamp(5));
    state.record_incoming_scan(timestamp(15));

    assert!(!state.active_session);
    assert_eq!(state.cargo_scans, 3);
    assert_eq!(state.last_scan_at, Some(timestamp(15)));
    assert_eq!(
        state.scan_timestamps(),
        &[timestamp(0), timestamp(5), timestamp(15)]
    );
    assert_eq!(state.total_scan_rate_per_hour_at(timestamp(60)), 0.0);
    assert_eq!(state.recent_scan_rate_per_hour_at(timestamp(10)), 12.0);
}

#[test]
fn session_state_public_api_can_be_driven_by_sanitized_parser_fixture() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join("journal_damage_fighter.log");
    let content = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", fixture_path.display()));
    let mut state = SessionState::new();

    for line in content.lines() {
        let event = parse_journal_line(line).unwrap();
        state.apply_event(&event);
    }

    assert_eq!(state.commander.as_deref(), Some("Cmdr Fixture Delta"));
    assert_eq!(state.ship.as_deref(), Some("krait_mkii"));
    assert_eq!(state.mode.as_deref(), Some("Solo"));
    assert_eq!(state.shields_up, Some(false));
    assert_eq!(state.ship_hull, Some(0.0));
    assert_eq!(state.fighter_alive, Some(false));
}
