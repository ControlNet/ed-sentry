use chrono::{TimeZone, Utc};
use ed_sentry::event::{parse_journal_line, JournalEvent, JournalParseError};

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
fn event_parser_public_api_preserves_raw_payload_for_typed_events() {
    let event = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:06:08Z","event":"Cargo","Vessel":"Ship","Count":7,"Inventory":[],"FixtureUnmodelledField":{"Nested":123}}"#,
    )
    .unwrap();

    assert!(matches!(event, JournalEvent::Cargo(_)));
    assert_eq!(
        event
            .raw_payload()
            .and_then(|raw| raw.get("FixtureUnmodelledField"))
            .and_then(|field| field.get("Nested")),
        Some(&serde_json::json!(123))
    );
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
fn event_parser_public_api_extracts_afk_risk_event_fields() {
    let interdicted = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:06:08Z","event":"Interdicted","Submitted":false,"Interdictor":"Fixture Interdictor","IsPlayer":false,"CombatRank":5}"#,
    )
    .unwrap();

    match interdicted {
        JournalEvent::Interdicted(event) => {
            assert_eq!(event.submitted, Some(false));
            assert_eq!(event.interdictor.as_deref(), Some("Fixture Interdictor"));
            assert_eq!(event.is_player, Some(false));
            assert_eq!(event.combat_rank, Some(5));
        }
        other => panic!("expected Interdicted, got {other:?}"),
    }

    let interdiction = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:07:08Z","event":"Interdiction","Success":true,"Interdicted":"Fixture Target","IsPlayer":false,"CombatRank":4,"Faction":"Fixture Faction","Power":"Fixture Power"}"#,
    )
    .unwrap();

    match interdiction {
        JournalEvent::Interdiction(event) => {
            assert_eq!(event.success, Some(true));
            assert_eq!(event.interdicted.as_deref(), Some("Fixture Target"));
            assert_eq!(event.is_player, Some(false));
            assert_eq!(event.combat_rank, Some(4));
            assert_eq!(event.faction.as_deref(), Some("Fixture Faction"));
            assert_eq!(event.power.as_deref(), Some("Fixture Power"));
        }
        other => panic!("expected Interdiction, got {other:?}"),
    }

    let escaped = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:08:08Z","event":"EscapeInterdiction","Interdictor":"Fixture Interdictor","IsPlayer":true}"#,
    )
    .unwrap();

    match escaped {
        JournalEvent::EscapeInterdiction(event) => {
            assert_eq!(event.interdictor.as_deref(), Some("Fixture Interdictor"));
            assert_eq!(event.is_player, Some(true));
        }
        other => panic!("expected EscapeInterdiction, got {other:?}"),
    }

    let under_attack = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:09:08Z","event":"UnderAttack","Target":"You"}"#,
    )
    .unwrap();

    match under_attack {
        JournalEvent::UnderAttack(event) => assert_eq!(event.target.as_deref(), Some("You")),
        other => panic!("expected UnderAttack, got {other:?}"),
    }
}

#[test]
fn event_parser_public_api_extracts_cargo_and_reward_ingest_fields() {
    let cargo = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:06:08Z","event":"Cargo","Vessel":"Ship","Count":7,"Inventory":[{"Name":"drones","Name_Localised":"Limpet","Count":5,"Stolen":0},{"Name":"gold","Count":2,"Stolen":1,"MissionID":42}]}"#,
    )
    .unwrap();

    match cargo {
        JournalEvent::Cargo(event) => {
            assert_eq!(event.vessel.as_deref(), Some("Ship"));
            assert_eq!(event.count, Some(7));
            assert_eq!(event.inventory.len(), 2);
            assert_eq!(event.inventory[0].name.as_deref(), Some("drones"));
            assert_eq!(event.inventory[0].name_localised.as_deref(), Some("Limpet"));
            assert_eq!(event.inventory[0].count, Some(5));
            assert_eq!(event.inventory[0].stolen, Some(0));
            assert_eq!(event.inventory[1].mission_id, Some(42));
        }
        other => panic!("expected Cargo, got {other:?}"),
    }

    let collect = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:07:08Z","event":"CollectCargo","Type":"gold","Type_Localised":"Gold","Count":2,"Stolen":false,"MissionID":42}"#,
    )
    .unwrap();

    match collect {
        JournalEvent::CollectCargo(event) => {
            assert_eq!(event.cargo_type.as_deref(), Some("gold"));
            assert_eq!(event.cargo_type_localised.as_deref(), Some("Gold"));
            assert_eq!(event.count, Some(2));
            assert_eq!(event.stolen, Some(false));
            assert_eq!(event.mission_id, Some(42));
        }
        other => panic!("expected CollectCargo, got {other:?}"),
    }

    let buy = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:08:08Z","event":"MarketBuy","Type":"drones","Type_Localised":"Limpet","Count":4,"BuyPrice":101,"TotalCost":404}"#,
    )
    .unwrap();

    match buy {
        JournalEvent::MarketBuy(event) => {
            assert_eq!(event.cargo_type.as_deref(), Some("drones"));
            assert_eq!(event.cargo_type_localised.as_deref(), Some("Limpet"));
            assert_eq!(event.count, Some(4));
            assert_eq!(event.buy_price, Some(101));
            assert_eq!(event.total_cost, Some(404));
        }
        other => panic!("expected MarketBuy, got {other:?}"),
    }

    let sell = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:09:08Z","event":"MarketSell","Type":"gold","Type_Localised":"Gold","Count":2,"SellPrice":12000,"TotalSale":24000,"AvgPricePaid":9000}"#,
    )
    .unwrap();

    match sell {
        JournalEvent::MarketSell(event) => {
            assert_eq!(event.cargo_type.as_deref(), Some("gold"));
            assert_eq!(event.cargo_type_localised.as_deref(), Some("Gold"));
            assert_eq!(event.count, Some(2));
            assert_eq!(event.sell_price, Some(12000));
            assert_eq!(event.total_sale, Some(24000));
            assert_eq!(event.avg_price_paid, Some(9000));
        }
        other => panic!("expected MarketSell, got {other:?}"),
    }

    let redeem = parse_journal_line(
        r#"{"timestamp":"2035-03-04T05:10:08Z","event":"RedeemVoucher","Type":"bounty","Amount":18000,"Faction":"Fixture Security","BrokerPercentage":25,"Factions":[{"Faction":"Fixture Security","Amount":12000},{"Faction":"Fixture Navy","Amount":6000}]}"#,
    )
    .unwrap();

    match redeem {
        JournalEvent::RedeemVoucher(event) => {
            assert_eq!(event.voucher_type.as_deref(), Some("bounty"));
            assert_eq!(event.amount, Some(18000));
            assert_eq!(event.faction.as_deref(), Some("Fixture Security"));
            assert_eq!(event.broker_percentage, Some(25));
            let factions = event.factions.as_ref().expect("faction breakdown");
            assert_eq!(factions.len(), 2);
            assert_eq!(factions[0].faction.as_deref(), Some("Fixture Security"));
            assert_eq!(factions[0].amount, Some(12000));
            assert_eq!(factions[1].faction.as_deref(), Some("Fixture Navy"));
            assert_eq!(factions[1].amount, Some(6000));
        }
        other => panic!("expected RedeemVoucher, got {other:?}"),
    }
}

#[test]
fn event_parser_public_api_accepts_sparse_cargo_ingest_events() {
    for line in [
        r#"{"timestamp":"2035-03-04T05:06:08Z","event":"Cargo"}"#,
        r#"{"timestamp":"2035-03-04T05:07:08Z","event":"CollectCargo"}"#,
        r#"{"timestamp":"2035-03-04T05:08:08Z","event":"MarketBuy"}"#,
        r#"{"timestamp":"2035-03-04T05:09:08Z","event":"MarketSell"}"#,
        r#"{"timestamp":"2035-03-04T05:10:08Z","event":"RedeemVoucher"}"#,
    ] {
        let event = parse_journal_line(line).unwrap();
        assert!(!matches!(event, JournalEvent::Unknown { .. }), "{event:?}");
    }
}

#[test]
fn event_parser_public_api_returns_recoverable_malformed_error() {
    let error =
        parse_journal_line(r#"{"timestamp":"2035-03-04T05:06:09Z","event":"MalformedFixture""#)
            .expect_err("malformed JSON should return an error");

    assert!(matches!(error, JournalParseError::MalformedJson { .. }));
}
