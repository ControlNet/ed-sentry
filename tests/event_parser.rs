use chrono::{TimeZone, Utc};
use ed_afk_dashboard::event::{parse_journal_line, JournalEvent, JournalParseError};

#[test]
fn event_parser_public_api_parses_known_event_for_downstream_callers() {
    let event = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:06:07Z","event":"ReceiveText","From":"npc_fixture_sender","Message":"Synthetic public API coverage.","Channel":"npc"}"#,
    )
    .unwrap();

    assert_eq!(event.event_name(), "ReceiveText");
    assert_eq!(
        event.timestamp(),
        Utc.with_ymd_and_hms(2035, 3, 4, 5, 6, 7).single().unwrap()
    );

    match event {
        JournalEvent::ReceiveText(receive_text) => {
            assert_eq!(receive_text.from.as_deref(), Some("npc_fixture_sender"));
            assert_eq!(
                receive_text.message.as_deref(),
                Some("Synthetic public API coverage.")
            );
            assert_eq!(receive_text.channel.as_deref(), Some("npc"));
        }
        other => panic!("expected ReceiveText, got {other:?}"),
    }
}

#[test]
fn event_parser_public_api_keeps_unknown_event_recoverable() {
    let event = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:06:08Z","event":"FixtureFutureEvent","FutureField":123}"#,
    )
    .unwrap();

    assert!(matches!(
        event,
        JournalEvent::Unknown {
            event: ref event_name,
            ..
        } if event_name == "FixtureFutureEvent"
    ));
    assert_eq!(
        event.raw_payload().and_then(|raw| raw.get("FutureField")),
        Some(&serde_json::json!(123))
    );
}

#[test]
fn event_parser_public_api_parses_broad_events_into_specific_typed_variants() {
    let event = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:06:08Z","event":"DockingGranted","LandingPad":42}"#,
    )
    .unwrap();

    assert!(matches!(event, JournalEvent::DockingGranted(_)));
    assert_eq!(event.event_name(), "DockingGranted");
    assert_eq!(
        event.raw_payload().and_then(|raw| raw.get("LandingPad")),
        Some(&serde_json::json!(42))
    );
}

#[test]
fn event_parser_public_api_extracts_mission_modeling_fields() {
    let event = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:06:08Z","event":"MissionAccepted","MissionID":42,"Name":"Mission_Delivery_name","Faction":"Fixture Logistics","DestinationSystem":"Delivery System","DestinationStation":"Delivery Port","Expiry":"2035-03-05T05:06:08Z","Influence":"Med","Reputation":"High","Reward":50000,"Wing":true,"Commodity":"syntheticfabric","Commodity_Localised":"Synthetic Fabric","Count":12}"#,
    )
    .unwrap();

    match event {
        JournalEvent::MissionAccepted(mission) => {
            assert_eq!(mission.mission_id, Some(42));
            assert_eq!(mission.faction.as_deref(), Some("Fixture Logistics"));
            assert_eq!(
                mission.destination_system.as_deref(),
                Some("Delivery System")
            );
            assert_eq!(
                mission.destination_station.as_deref(),
                Some("Delivery Port")
            );
            assert_eq!(mission.influence.as_deref(), Some("Med"));
            assert_eq!(mission.reputation.as_deref(), Some("High"));
            assert_eq!(mission.reward, Some(50000));
            assert_eq!(mission.wing, Some(true));
            assert_eq!(mission.commodity.as_deref(), Some("syntheticfabric"));
            assert_eq!(
                mission.commodity_localised.as_deref(),
                Some("Synthetic Fabric")
            );
            assert_eq!(mission.count, Some(12));
            assert_eq!(
                mission.expiry,
                Some(Utc.with_ymd_and_hms(2035, 3, 5, 5, 6, 8).single().unwrap())
            );
        }
        other => panic!("expected MissionAccepted, got {other:?}"),
    }

    let cargo = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:07:08Z","event":"CargoDepot","MissionID":42,"CargoType":"syntheticfabric","CargoType_Localised":"Synthetic Fabric","ItemsCollected":12,"ItemsDelivered":5,"TotalItemsToDeliver":12,"Progress":0.42}"#,
    )
    .unwrap();

    match cargo {
        JournalEvent::CargoDepot(cargo) => {
            assert_eq!(cargo.mission_id, Some(42));
            assert_eq!(cargo.cargo_type.as_deref(), Some("syntheticfabric"));
            assert_eq!(cargo.items_collected, Some(12));
            assert_eq!(cargo.items_delivered, Some(5));
            assert_eq!(cargo.total_items_to_deliver, Some(12));
        }
        other => panic!("expected CargoDepot, got {other:?}"),
    }
}

#[test]
fn event_parser_public_api_returns_recoverable_malformed_error() {
    let error =
        parse_journal_line(r#"{"timestamp":"2035-03-04T05:06:09Z","event":"MalformedFixture""#)
            .expect_err("malformed JSON should return an error");

    assert!(matches!(error, JournalParseError::MalformedJson { .. }));
}
