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
fn event_parser_public_api_returns_recoverable_malformed_error() {
    let error =
        parse_journal_line(r#"{"timestamp":"2035-03-04T05:06:09Z","event":"MalformedFixture""#)
            .expect_err("malformed JSON should return an error");

    assert!(matches!(error, JournalParseError::MalformedJson { .. }));
}
