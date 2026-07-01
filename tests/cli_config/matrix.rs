use serde_json::Value;
use std::io::Write;
use std::time::{Duration, Instant};

const MATRIX_RECORD_POLL_INTERVAL: Duration = Duration::from_millis(20);

pub fn append_journal_lines(path: &std::path::Path, lines: &str) {
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(lines.as_bytes()).unwrap();
    file.flush().unwrap();
}

pub fn write_matrix_config(
    path: &std::path::Path,
    journal_folder: &std::path::Path,
    live_status: bool,
) {
    std::fs::write(
        path,
        format!(
            r#"
            [journal]
            folder = {:?}

            [monitor]
            live_status = {}
            poll_interval_ms = 1000

            [matrix]
            enabled = true
            homeserver = "https://matrix.invalid"
            room_id = "!room:matrix.invalid"
            access_{} = "fixture-access"
            mention_user_id = "@commander:matrix.invalid"
            status_update_interval_seconds = 60
            "#,
            journal_folder.display().to_string(),
            live_status,
            "token",
        ),
    )
    .unwrap();
}

#[test]
fn write_matrix_config_escapes_windows_journal_folder_paths() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    let journal_folder = std::path::Path::new(
        r"C:\Users\Commander\Saved Games\Frontier Developments\Elite Dangerous",
    );

    write_matrix_config(&config_path, journal_folder, true);

    let config = std::fs::read_to_string(&config_path).unwrap();
    let parsed: toml::Value = toml::from_str(&config).unwrap();
    assert_eq!(
        parsed["journal"]["folder"].as_str(),
        Some(journal_folder.to_str().unwrap())
    );
}

pub fn read_matrix_records(path: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

pub fn is_live_cobra_send_record(record: &Value) -> bool {
    record["kind"] == "send"
        && record["remote_text"]
            .as_str()
            .is_some_and(|text| text.contains("Kill: Live Cobra"))
}

pub fn wait_for_matrix_record(
    path: &std::path::Path,
    matches_record: impl Fn(&Value) -> bool,
    deadline: Duration,
) -> Vec<Value> {
    let started = Instant::now();
    while started.elapsed() < deadline {
        let records = if path.exists() {
            read_matrix_records(path)
        } else {
            Vec::new()
        };
        if records.iter().any(&matches_record) {
            return records;
        }
        std::thread::sleep(MATRIX_RECORD_POLL_INTERVAL);
    }
    let records = if path.exists() {
        read_matrix_records(path)
    } else {
        Vec::new()
    };
    panic!("timed out waiting for expected fake Matrix record; observed records: {records:?}");
}
