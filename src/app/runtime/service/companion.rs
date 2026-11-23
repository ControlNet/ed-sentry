use std::{fs, path::Path, path::PathBuf};

use super::super::file_watcher::resolve_and_normalize_path;
use crate::app::AfkChecklistState;

const STATUS_FILE: &str = "Status.json";
const CARGO_FILE: &str = "Cargo.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompanionFile {
    Status,
    Cargo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CompanionPaths {
    status: PathBuf,
    cargo: PathBuf,
}

impl CompanionPaths {
    pub(super) fn from_journal_file(journal_file: &Path) -> Option<Self> {
        let resolved_journal = resolve_and_normalize_path(journal_file);
        let companion_dir = resolved_journal
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| resolve_and_normalize_path(Path::new(".")));
        Some(Self {
            status: resolve_and_normalize_path(&companion_dir.join(STATUS_FILE)),
            cargo: resolve_and_normalize_path(&companion_dir.join(CARGO_FILE)),
        })
    }

    pub(super) fn startup_state(&self) -> AfkChecklistState {
        let status_json = read_optional_companion_json(&self.status);
        let cargo_json = read_optional_companion_json(&self.cargo);

        AfkChecklistState::from_optional_companion_json(
            status_json.as_deref(),
            cargo_json.as_deref(),
        )
    }

    pub(super) fn refresh_path(&self, state: &mut AfkChecklistState, path: &Path) -> bool {
        match self.classify(path) {
            Some(CompanionFile::Status) => self.refresh_file(state, CompanionFile::Status),
            Some(CompanionFile::Cargo) => self.refresh_file(state, CompanionFile::Cargo),
            None => false,
        }
    }

    pub(super) fn refresh_file(&self, state: &mut AfkChecklistState, file: CompanionFile) -> bool {
        match file {
            CompanionFile::Status => {
                let status_json = read_optional_companion_json(&self.status);
                state.apply_status_json(status_json.as_deref())
            }
            CompanionFile::Cargo => {
                let cargo_json = read_optional_companion_json(&self.cargo);
                state.apply_cargo_json(cargo_json.as_deref())
            }
        }
    }

    fn classify(&self, path: &Path) -> Option<CompanionFile> {
        let normalized = resolve_and_normalize_path(path);
        if normalized == self.status {
            return Some(CompanionFile::Status);
        }
        if normalized == self.cargo {
            return Some(CompanionFile::Cargo);
        }
        None
    }
}

fn read_optional_companion_json(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ChecklistRowState;

    #[test]
    fn companion_paths_from_relative_journal_match_absolute_notify_paths() {
        let cwd = std::env::current_dir().unwrap();
        let relative = Path::new("relative-journals/Journal.2035-01-04T120000.01.log");
        let paths = CompanionPaths::from_journal_file(relative).unwrap();
        let abs_dir = resolve_and_normalize_path(&cwd.join("relative-journals"));

        assert_eq!(
            paths.classify(&abs_dir.join(STATUS_FILE)),
            Some(CompanionFile::Status)
        );
        assert_eq!(
            paths.classify(&abs_dir.join(CARGO_FILE)),
            Some(CompanionFile::Cargo)
        );
        assert_eq!(
            paths.classify(Path::new("relative-journals/Status.json")),
            Some(CompanionFile::Status)
        );
    }

    #[test]
    fn companion_paths_from_bare_journal_match_absolute_cwd_paths() {
        let cwd = std::env::current_dir().unwrap();
        let paths =
            CompanionPaths::from_journal_file(Path::new("Journal.2035-01-04T120000.01.log"))
                .unwrap();
        let abs_cwd = resolve_and_normalize_path(&cwd);

        assert_eq!(
            paths.classify(&abs_cwd.join(STATUS_FILE)),
            Some(CompanionFile::Status)
        );
        assert_eq!(
            paths.classify(&abs_cwd.join(CARGO_FILE)),
            Some(CompanionFile::Cargo)
        );
        let state = AfkChecklistState::unknown();
        assert_eq!(state.to_view().rows[0].state, ChecklistRowState::Unknown);
    }
}
