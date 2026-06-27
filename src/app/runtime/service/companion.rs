use std::{fs, path::Path, path::PathBuf};

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
        let companion_dir = journal_file.parent()?;
        Some(Self {
            status: companion_dir.join(STATUS_FILE),
            cargo: companion_dir.join(CARGO_FILE),
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
        if path == self.status {
            return Some(CompanionFile::Status);
        }
        if path == self.cargo {
            return Some(CompanionFile::Cargo);
        }
        None
    }
}

fn read_optional_companion_json(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}
