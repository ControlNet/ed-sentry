use toml::Value;

use super::{read_string, read_u16};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalConfig {
    pub folder: String,
    pub recent_files: u16,
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            folder: String::new(),
            recent_files: 10,
        }
    }
}

pub(super) fn read_journal_config(
    table: &toml::map::Map<String, Value>,
    journal: &mut JournalConfig,
    warnings: &mut Vec<String>,
) {
    read_string(
        table.get("folder"),
        "journal.folder",
        &mut journal.folder,
        warnings,
    );
    read_u16(
        table.get("recent_files"),
        "journal.recent_files",
        &mut journal.recent_files,
        warnings,
    );
}
