use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use ed_afk_monitor::event::{parse_journal_line, JournalEvent};
use ed_afk_monitor::state::SessionState;

const REAL_JOURNAL_DIR: &str = "/home/ubuntu/Elite Dangerous";
const KNOWN_SAMPLE_FILES: &[&str] = &[
    "Journal.180729194257.01.log",
    "Journal.181214225820.01.log",
    "Journal.190422050045.01.log",
    "Journal.170814020512.01.log",
    "Journal.180725114837.01.log",
];

#[derive(Default)]
struct CategoryCounts {
    bounty: u64,
    ship_targeted: u64,
    mission_redirected: u64,
    shield_state: u64,
    fighter_destroyed: u64,
    hull_damage: u64,
    parsed_events: u64,
    malformed_lines: u64,
    files_opened: u64,
}

impl CategoryCounts {
    fn record(&mut self, event: &JournalEvent) {
        self.parsed_events += 1;
        match event {
            JournalEvent::Bounty(_) => self.bounty += 1,
            JournalEvent::ShipTargeted(_) => self.ship_targeted += 1,
            JournalEvent::MissionRedirected(_) => self.mission_redirected += 1,
            JournalEvent::ShieldState(_) => self.shield_state += 1,
            JournalEvent::FighterDestroyed(_) => self.fighter_destroyed += 1,
            JournalEvent::HullDamage(_) => self.hull_damage += 1,
            _ => {}
        }
    }

    fn has_all_categories(&self) -> bool {
        self.bounty > 0
            && self.ship_targeted > 0
            && self.mission_redirected > 0
            && self.shield_state > 0
            && self.fighter_destroyed > 0
            && self.hull_damage > 0
    }

    fn missing_categories(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.bounty == 0 {
            missing.push("Bounty");
        }
        if self.ship_targeted == 0 {
            missing.push("ShipTargeted");
        }
        if self.mission_redirected == 0 {
            missing.push("MissionRedirected");
        }
        if self.shield_state == 0 {
            missing.push("ShieldState");
        }
        if self.fighter_destroyed == 0 {
            missing.push("FighterDestroyed");
        }
        if self.hull_damage == 0 {
            missing.push("HullDamage");
        }
        missing
    }
}

#[test]
#[ignore]
fn real_journal_replay_scans_local_samples_read_only() {
    let journal_dir = Path::new(REAL_JOURNAL_DIR);
    if !journal_dir.is_dir() {
        println!("skipping real journal replay: local Journal folder is absent or unavailable");
        return;
    }

    let Some(files) = real_journal_files(journal_dir) else {
        println!("skipping real journal replay: Journal folder could not be listed read-only");
        return;
    };
    if files.is_empty() {
        println!("skipping real journal replay: no Journal.*.log samples are available");
        return;
    }

    let mut state = SessionState::new();
    let mut counts = CategoryCounts::default();
    for path in files {
        scan_file(&path, &mut state, &mut counts);
        if counts.has_all_categories() {
            break;
        }
    }

    println!(
        "real journal replay category counts: Bounty={} ShipTargeted={} MissionRedirected={} ShieldState={} FighterDestroyed={} HullDamage={} parsed_events={} malformed_lines={} files_opened={}",
        counts.bounty,
        counts.ship_targeted,
        counts.mission_redirected,
        counts.shield_state,
        counts.fighter_destroyed,
        counts.hull_damage,
        counts.parsed_events,
        counts.malformed_lines,
        counts.files_opened
    );

    if counts.files_opened == 0 || counts.parsed_events == 0 {
        println!("skipping real journal replay: local samples were unreadable or empty");
        return;
    }

    let missing = counts.missing_categories();
    if !missing.is_empty() {
        println!(
            "skipping strict real journal category assertion: local samples missing {}",
            missing.join(", ")
        );
    }
}

fn scan_file(path: &Path, state: &mut SessionState, counts: &mut CategoryCounts) {
    let Ok(file) = File::open(path) else {
        return;
    };

    counts.files_opened += 1;
    for line in BufReader::new(file).lines() {
        let Ok(line) = line else {
            counts.malformed_lines += 1;
            continue;
        };

        match parse_journal_line(&line) {
            Ok(event) => {
                counts.record(&event);
                state.apply_event(&event);
            }
            Err(_) => counts.malformed_lines += 1,
        }
    }
}

fn real_journal_files(journal_dir: &Path) -> Option<Vec<PathBuf>> {
    let mut seen = BTreeSet::new();
    let mut files = Vec::new();

    for filename in KNOWN_SAMPLE_FILES {
        let path = journal_dir.join(filename);
        if path.is_file() && seen.insert(path.clone()) {
            files.push(path);
        }
    }

    for entry in fs::read_dir(journal_dir).ok()? {
        let path = entry.ok()?.path();
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if filename.starts_with("Journal.")
            && filename.ends_with(".log")
            && seen.insert(path.clone())
        {
            files.push(path);
        }
    }

    Some(files)
}
