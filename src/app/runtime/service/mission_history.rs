use std::path::{Path, PathBuf};

use crate::config::RuntimeConfig;
use crate::event::parse_journal_line;
use crate::journal::{
    discover_journal_files, discover_runtime_journal_files, preload_journal_file, JournalFile,
};
use crate::mission::MissionTracker;

use super::super::RuntimeError;

pub(super) fn preload_mission_history(
    config: &RuntimeConfig,
    selected_file: &Path,
) -> Result<MissionTracker, RuntimeError> {
    let mut tracker = MissionTracker::new();
    for path in mission_history_paths(config, selected_file)? {
        let preload = preload_journal_file(path, parse_journal_line)?;
        for record in preload.records {
            if let Ok(event) = record.result {
                tracker.apply_event(&event);
            }
        }
    }
    Ok(tracker)
}

fn mission_history_paths(
    config: &RuntimeConfig,
    selected_file: &Path,
) -> Result<Vec<PathBuf>, RuntimeError> {
    let files = discover_history_candidates(config, selected_file)?;
    let Some(selected_index) = files
        .iter()
        .position(|file| paths_match(&file.path, selected_file))
    else {
        return Ok(Vec::new());
    };

    let mut paths: Vec<_> = files
        .into_iter()
        .skip(selected_index + 1)
        .take(usize::from(config.journal.recent_files))
        .map(|file| file.path)
        .collect();
    paths.reverse();
    Ok(paths)
}

fn discover_history_candidates(
    config: &RuntimeConfig,
    selected_file: &Path,
) -> Result<Vec<JournalFile>, RuntimeError> {
    if config.set_file.is_some() {
        let Some(parent) = selected_file.parent() else {
            return Ok(Vec::new());
        };
        return discover_journal_files(parent).map_err(RuntimeError::from);
    }

    discover_runtime_journal_files(config).map_err(RuntimeError::from)
}

fn paths_match(candidate: &Path, selected_file: &Path) -> bool {
    if candidate == selected_file {
        return true;
    }

    match (candidate.canonicalize(), selected_file.canonicalize()) {
        (Ok(candidate), Ok(selected_file)) => candidate == selected_file,
        _ => false,
    }
}
