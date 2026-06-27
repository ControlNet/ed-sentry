use super::*;

fn checklist(status_json: &str, cargo_json: &str) -> AfkChecklistView {
    AfkChecklistState::from_companion_json(status_json, cargo_json).to_view()
}

fn row<'a>(view: &'a AfkChecklistView, id: &str) -> &'a AfkChecklistRowView {
    view.rows.iter().find(|row| row.id == id).unwrap()
}

#[test]
fn hardpoints_deployed_passes_when_status_flags_has_bit_0x40() {
    let view = checklist(
        r#"{ "Flags": 64, "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );

    assert_eq!(
        row(&view, "hardpoints_deployed").state,
        ChecklistRowState::Pass
    );
    assert_eq!(
        row(&view, "hardpoints_deployed").source,
        ChecklistRowSource::StatusJson
    );
}

#[test]
fn hardpoints_deployed_fails_when_status_flags_lacks_bit_0x40() {
    let view = checklist(
        r#"{ "Flags": 0, "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );

    assert_eq!(
        row(&view, "hardpoints_deployed").state,
        ChecklistRowState::Fail
    );
}

#[test]
fn hardpoints_deployed_is_unknown_when_flags_absent_or_malformed() {
    let absent = checklist(
        r#"{ "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );
    let malformed = checklist(
        r#"{ "Flags": "deployed", "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );

    assert_eq!(
        row(&absent, "hardpoints_deployed").state,
        ChecklistRowState::Unknown
    );
    assert_eq!(
        row(&malformed, "hardpoints_deployed").state,
        ChecklistRowState::Unknown
    );
}

#[test]
fn engine_pips_zero_passes_when_engine_pips_are_zero() {
    let view = checklist(
        r#"{ "Flags": 64, "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );

    assert_eq!(
        row(&view, "engine_pips_zero").state,
        ChecklistRowState::Pass
    );
    assert_eq!(
        row(&view, "engine_pips_zero").source,
        ChecklistRowSource::StatusJson
    );
}

#[test]
fn engine_pips_zero_fails_when_engine_pips_are_positive() {
    let view = checklist(
        r#"{ "Flags": 64, "Pips": [4, 2, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );

    assert_eq!(
        row(&view, "engine_pips_zero").state,
        ChecklistRowState::Fail
    );
}

#[test]
fn engine_pips_zero_is_unknown_when_pips_missing_short_or_malformed() {
    let missing = checklist(
        r#"{ "Flags": 64 }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );
    let short = checklist(
        r#"{ "Flags": 64, "Pips": [4] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );
    let malformed = checklist(
        r#"{ "Flags": 64, "Pips": "balanced" }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );

    assert_eq!(
        row(&missing, "engine_pips_zero").state,
        ChecklistRowState::Unknown
    );
    assert_eq!(
        row(&short, "engine_pips_zero").state,
        ChecklistRowState::Unknown
    );
    assert_eq!(
        row(&malformed, "engine_pips_zero").state,
        ChecklistRowState::Unknown
    );
}

#[test]
fn cargo_loaded_passes_for_ship_count_or_inventory() {
    let count = checklist(
        r#"{ "Flags": 64, "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 1, "Inventory": [] }"#,
    );
    let inventory = checklist(
        r#"{ "Flags": 64, "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [{ "Name": "bertrandite", "Count": 1 }] }"#,
    );

    assert_eq!(row(&count, "cargo_loaded").state, ChecklistRowState::Pass);
    assert_eq!(
        row(&inventory, "cargo_loaded").state,
        ChecklistRowState::Pass
    );
    assert_eq!(
        row(&count, "cargo_loaded").source,
        ChecklistRowSource::CargoJson
    );
}

#[test]
fn cargo_loaded_fails_for_ship_with_empty_cargo() {
    let view = checklist(
        r#"{ "Flags": 64, "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "Ship", "Count": 0, "Inventory": [] }"#,
    );

    assert_eq!(row(&view, "cargo_loaded").state, ChecklistRowState::Fail);
}

#[test]
fn cargo_loaded_is_unknown_for_non_ship_vessel() {
    let view = checklist(
        r#"{ "Flags": 64, "Pips": [4, 0, 8] }"#,
        r#"{ "Vessel": "SRV", "Count": 1, "Inventory": [{ "Name": "bertrandite" }] }"#,
    );

    assert_eq!(row(&view, "cargo_loaded").state, ChecklistRowState::Unknown);
    assert_eq!(
        row(&view, "cargo_loaded").source,
        ChecklistRowSource::Unknown
    );
}

#[test]
fn malformed_json_makes_rows_unknown_without_panic() {
    let view = checklist(r#"{ "Flags": "#, r#"{ "Vessel": "Ship", "Count": "#);

    assert_eq!(
        row(&view, "hardpoints_deployed").state,
        ChecklistRowState::Unknown
    );
    assert_eq!(
        row(&view, "engine_pips_zero").state,
        ChecklistRowState::Unknown
    );
    assert_eq!(row(&view, "cargo_loaded").state, ChecklistRowState::Unknown);
}
