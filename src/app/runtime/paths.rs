use std::path::{Path, PathBuf};

use crate::config::RuntimeConfig;
use crate::event::JournalEvent;
use crate::journal::PreloadRecord;
use crate::journal::{default_journal_folder, JournalFileChoice};
use crate::text::line_safe;

use super::RuntimeError;

pub fn journal_basename(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

pub fn watch_journal_folder_display(config: &RuntimeConfig) -> String {
    if !config.journal.folder.is_empty() {
        return config.journal.folder.clone();
    }

    default_journal_folder()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "<Windows Saved Games known folder unavailable>".to_string())
}

pub fn snapshot_journal_folder_display(config: &RuntimeConfig) -> String {
    if config.journal.folder.is_empty() {
        "Default journal folder".to_string()
    } else {
        "Configured journal folder".to_string()
    }
}

pub fn matrix_validation_reason(warning: &str) -> String {
    warning
        .strip_prefix("Matrix delivery disabled for this run: ")
        .unwrap_or(warning)
        .to_string()
}

pub fn redact_matrix_error_message(error: &anyhow::Error, access_token: &str) -> String {
    let message = line_safe(&error.to_string());
    if access_token.is_empty() {
        message
    } else {
        message.replace(access_token, "<redacted>")
    }
}

pub fn selected_journal_from_choices(
    choices: Vec<JournalFileChoice>,
    selected: usize,
) -> Result<PathBuf, RuntimeError> {
    choices
        .into_iter()
        .find(|choice| choice.number == selected)
        .map(|choice| choice.file.path)
        .ok_or_else(|| RuntimeError::new(format!("Journal selection {selected} is out of range")))
}

pub fn startup_commander(records: &[PreloadRecord<JournalEvent>]) -> Option<String> {
    records
        .iter()
        .find_map(|record| match record.result.as_ref().ok()? {
            JournalEvent::Commander(event) => event.name.clone(),
            JournalEvent::LoadGame(event) => event.commander.clone(),
            _ => None,
        })
}
