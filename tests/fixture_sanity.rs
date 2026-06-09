use std::fs;
use std::path::Path;

const FIXTURES: &[&str] = &[
    "journal_minimal_start.log",
    "journal_combat_bounty.log",
    "journal_missions.log",
    "journal_damage_fighter.log",
    "journal_malformed_unknown.log",
    "journal_warning_clock.log",
];

const MALFORMED_FIXTURE: &str = "journal_malformed_unknown.log";
const EXPECTED_MALFORMED_LINES: usize = 1;
const FORBIDDEN_FIXTURE_SUBSTRINGS: &[&str] = &[
    "/home/ubuntu/Elite Dangerous",
    "access_token",
    "password",
    "BEGIN ",
    "PRIVATE KEY",
];

#[test]
fn fixture_sanity_all_journal_lines_are_json_except_deliberate_malformed_line() {
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut malformed_lines = 0;

    for fixture in FIXTURES {
        let path = fixture_dir.join(fixture);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

        assert!(!content.trim().is_empty(), "{} is empty", path.display());
        assert!(
            content.ends_with('\n'),
            "{} must end with newline",
            path.display()
        );

        for forbidden in FORBIDDEN_FIXTURE_SUBSTRINGS {
            assert!(
                !content.contains(forbidden),
                "{} contains forbidden substring {forbidden:?}",
                path.display()
            );
        }

        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                panic!("{}:{} is blank", path.display(), line_index + 1);
            }

            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(value) => {
                    assert_eq!(
                        value.get("timestamp").and_then(serde_json::Value::as_str),
                        Some(value["timestamp"].as_str().unwrap_or_default()),
                        "{}:{} missing string timestamp",
                        path.display(),
                        line_index + 1
                    );
                    assert!(
                        value
                            .get("event")
                            .and_then(serde_json::Value::as_str)
                            .is_some(),
                        "{}:{} missing string event",
                        path.display(),
                        line_index + 1
                    );
                }
                Err(error) if *fixture == MALFORMED_FIXTURE => {
                    assert!(
                        line.contains("MalformedFixture"),
                        "{}:{} unexpected malformed line: {error}",
                        path.display(),
                        line_index + 1
                    );
                    malformed_lines += 1;
                }
                Err(error) => panic!(
                    "{}:{} should be valid JSON: {error}",
                    path.display(),
                    line_index + 1
                ),
            }
        }
    }

    assert_eq!(malformed_lines, EXPECTED_MALFORMED_LINES);
}

#[test]
fn fixture_sanity_malformed_fixture_contains_unknown_valid_event() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(MALFORMED_FIXTURE);
    let content = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", fixture_path.display()));

    let has_unknown_valid_event = content
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .any(|value| {
            value.get("event").and_then(serde_json::Value::as_str) == Some("FixtureUnknownEvent")
        });

    assert!(
        has_unknown_valid_event,
        "{} must contain one valid unknown event",
        fixture_path.display()
    );
}
