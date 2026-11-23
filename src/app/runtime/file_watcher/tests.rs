use std::cell::Cell;
use std::fs;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn afk_watcher_normalizes_selected_journal_path_event() {
    let temp_dir = tempfile::tempdir().unwrap();
    let selected = temp_dir.path().join("Journal.2035-01-04T120000.01.log");
    fs::write(&selected, "").unwrap();
    let paths = WatchedFileSet::new(&selected);

    let event = paths.classify_path(
        temp_dir
            .path()
            .join(".")
            .join(selected.file_name().unwrap()),
    );

    assert_eq!(
        event,
        Some(AfkWatcherEvent::SelectedFile {
            path: selected.clone()
        })
    );
    println!(
        "afk_watcher selected_file_event={}",
        selected.file_name().unwrap().to_string_lossy()
    );
}

#[test]
fn afk_watcher_normalizes_status_and_cargo_events() {
    let temp_dir = tempfile::tempdir().unwrap();
    let selected = temp_dir.path().join("Journal.2035-01-04T120000.01.log");
    let paths = WatchedFileSet::new(&selected);

    let status = paths.classify_path(temp_dir.path().join("Status.json"));
    let cargo = paths.classify_path(temp_dir.path().join("Cargo.json"));

    assert_eq!(
        status,
        Some(AfkWatcherEvent::StatusJson {
            path: temp_dir.path().join("Status.json")
        })
    );
    assert_eq!(
        cargo,
        Some(AfkWatcherEvent::CargoJson {
            path: temp_dir.path().join("Cargo.json")
        })
    );
    println!("afk_watcher companion_events=status,cargo");
}

#[test]
fn afk_watcher_ignores_unselected_journal_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let selected = temp_dir.path().join("Journal.2035-01-04T120000.01.log");
    let paths = WatchedFileSet::new(&selected);

    let event = paths.classify_path(temp_dir.path().join("Journal.2035-01-04T121000.01.log"));

    assert_eq!(event, None);
    println!("afk_watcher ignored_unselected_journal=true");
}

#[test]
fn afk_watcher_accepts_explicit_non_journal_selected_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let selected = temp_dir.path().join("fixture.log");
    let paths = WatchedFileSet::new(&selected);

    let selected_event = paths.classify_path(&selected);
    let journal_event =
        paths.classify_path(temp_dir.path().join("Journal.2035-01-04T121000.01.log"));

    assert_eq!(
        selected_event,
        Some(AfkWatcherEvent::SelectedFile {
            path: selected.clone()
        })
    );
    assert_eq!(journal_event, None);
    println!("afk_watcher explicit_non_journal_selected=true");
}

#[test]
fn afk_watcher_start_watches_selected_file_parent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let selected = temp_dir.path().join("fixture.log");
    fs::write(&selected, "").unwrap();

    let start = AfkFileWatcherStart::start(&selected);

    match start {
        AfkFileWatcherStart::Watching { watcher, events: _ } => {
            assert_eq!(watcher.watch_dir(), temp_dir.path());
            println!("afk_watcher mode=watching watch_dir_is_parent=true");
        }
        AfkFileWatcherStart::PollingFallback { warning } => {
            panic!(
                "expected watcher to start, got fallback: {}",
                warning.message()
            )
        }
    }
}

#[test]
fn afk_watcher_init_failure_degrades_to_polling_warning() {
    let selected = std::env::temp_dir()
        .join(format!(
            "ed-sentry-missing-watch-parent-{}",
            std::process::id()
        ))
        .join("Journal.2035-01-04T120000.01.log");

    let start = AfkFileWatcherStart::start(&selected);

    match start {
        AfkFileWatcherStart::PollingFallback { warning } => {
            assert!(warning.message().contains("polling fallback"));
            assert!(!warning.message().contains("\n"));
            assert!(!warning.message().contains(&selected.display().to_string()));
            println!("afk_watcher fallback_warning={}", warning.message());
        }
        AfkFileWatcherStart::Watching { .. } => panic!("expected polling fallback"),
    }
}

#[test]
fn afk_watcher_debounce_coalesces_duplicate_companion_events() {
    let temp_dir = tempfile::tempdir().unwrap();
    let status = temp_dir.path().join("Status.json");
    let event = AfkWatcherEvent::StatusJson { path: status };
    let mut debounce = DebouncedWatcherEvents::new(Duration::from_millis(50));
    let start = Instant::now();

    debounce.push(event.clone(), start);
    debounce.push(event, start + Duration::from_millis(10));

    assert!(debounce
        .drain_ready(start + Duration::from_millis(49))
        .is_empty());
    let ready = debounce.drain_ready(start + Duration::from_millis(60));

    assert_eq!(ready.len(), 1);
    assert!(matches!(ready[0], AfkWatcherEvent::StatusJson { .. }));
    println!("afk_watcher debounce_ready_count={}", ready.len());
}

#[test]
fn afk_companion_update_retries_partial_json_then_publishes_valid_state() {
    let attempts = Cell::new(0_u8);
    let retry = CompanionReadRetry::new(3);

    let value: serde_json::Value = retry
        .read(|| {
            let attempt = attempts.get() + 1;
            attempts.set(attempt);
            if attempt == 1 {
                return Err(CompanionReadFailure::Retryable("partial JSON".to_string()));
            }
            serde_json::from_str(r#"{"Flags":64,"Pips":[4,0,8]}"#)
                .map_err(|error| CompanionReadFailure::Final(error.to_string()))
        })
        .unwrap();

    assert_eq!(attempts.get(), 2);
    assert_eq!(value["Flags"], 64);
    println!(
        "afk_companion_update retry_attempts={} accepted_flags={}",
        attempts.get(),
        value["Flags"]
    );
}

#[test]
fn afk_watcher_bare_selected_filename_matches_companion_events() {
    let cwd = std::env::current_dir().unwrap();
    let paths = WatchedFileSet::new(Path::new("Journal.2035-01-04T120000.01.log"));
    let expected_selected = normalize_path(&cwd.join("Journal.2035-01-04T120000.01.log"));
    let expected_status = normalize_path(&cwd.join("Status.json"));
    let expected_cargo = normalize_path(&cwd.join("Cargo.json"));

    assert_eq!(
        paths.classify_path(&expected_selected),
        Some(AfkWatcherEvent::SelectedFile {
            path: expected_selected.clone(),
        })
    );
    assert_eq!(
        paths.classify_path(Path::new("Status.json")),
        Some(AfkWatcherEvent::StatusJson {
            path: expected_status.clone(),
        })
    );
    assert_eq!(
        paths.classify_path(Path::new("./Status.json")),
        Some(AfkWatcherEvent::StatusJson {
            path: expected_status.clone(),
        })
    );
    assert_eq!(
        paths.classify_path(&expected_status),
        Some(AfkWatcherEvent::StatusJson {
            path: expected_status,
        })
    );
    assert_eq!(
        paths.classify_path(Path::new("Cargo.json")),
        Some(AfkWatcherEvent::CargoJson {
            path: expected_cargo.clone(),
        })
    );
    assert_eq!(
        paths.classify_path(Path::new("./Cargo.json")),
        Some(AfkWatcherEvent::CargoJson {
            path: expected_cargo.clone(),
        })
    );
    assert_eq!(
        paths.classify_path(&expected_cargo),
        Some(AfkWatcherEvent::CargoJson {
            path: expected_cargo,
        })
    );
}

#[test]
fn afk_watcher_relative_folder_matches_absolute_notify_event_paths() {
    let cwd = std::env::current_dir().unwrap();
    let relative_journal = Path::new("relative-journals/Journal.2035-01-04T120000.01.log");
    let paths = WatchedFileSet::new(relative_journal);
    let abs_dir = normalize_path(&cwd.join("relative-journals"));

    assert_eq!(
        paths.classify_path(abs_dir.join("Journal.2035-01-04T120000.01.log")),
        Some(AfkWatcherEvent::SelectedFile {
            path: abs_dir.join("Journal.2035-01-04T120000.01.log"),
        })
    );
    assert_eq!(
        paths.classify_path(abs_dir.join("Status.json")),
        Some(AfkWatcherEvent::StatusJson {
            path: abs_dir.join("Status.json"),
        })
    );
    assert_eq!(
        paths.classify_path(abs_dir.join("Cargo.json")),
        Some(AfkWatcherEvent::CargoJson {
            path: abs_dir.join("Cargo.json"),
        })
    );
}

#[tokio::test]
async fn afk_watcher_real_watcher_emits_events_for_relative_journal_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let abs_journal = temp_dir.path().join("Journal.2035-01-04T120000.01.log");
    fs::write(&abs_journal, "").unwrap();
    let cwd = std::env::current_dir().unwrap();
    let relative_journal =
        relative_path_from(&cwd, &abs_journal).unwrap_or_else(|| abs_journal.clone());

    let start = AfkFileWatcherStart::start(&relative_journal);
    let AfkFileWatcherStart::Watching {
        watcher: _watcher,
        mut events,
    } = start
    else {
        panic!("expected watcher to start for relative journal path");
    };

    let status_path = temp_dir.path().join("Status.json");
    fs::write(&status_path, r#"{"Flags":64,"Pips":[4,0,8]}"#).unwrap();

    let received = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            match events.recv().await {
                Some(AfkWatcherEvent::StatusJson { path }) => return path,
                Some(_) => continue,
                None => panic!("watcher closed before emitting StatusJson"),
            }
        }
    })
    .await
    .expect("watcher should emit StatusJson for relative journal path");

    let expected_status = normalize_path(&status_path);
    assert_eq!(received, expected_status);
}

fn relative_path_from(base: &Path, target: &Path) -> Option<PathBuf> {
    let base = normalize_path(base);
    let target = normalize_path(target);
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();
    if base_components.first() != target_components.first() {
        return None;
    }
    let common_len = base_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut relative = PathBuf::new();
    for _ in common_len..base_components.len() {
        relative.push("..");
    }
    for component in &target_components[common_len..] {
        relative.push(component.as_os_str());
    }
    Some(relative)
}
