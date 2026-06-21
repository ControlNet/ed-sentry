use std::path::PathBuf;

use super::{ConfigSource, JournalConfig, LogLevelConfig, MatrixConfig, MonitorConfig, WebConfig};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppConfig {
    pub journal: JournalConfig,
    pub monitor: MonitorConfig,
    pub log_levels: LogLevelConfig,
    pub matrix: Option<MatrixConfig>,
    pub web: WebConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliConfigOverrides {
    pub journal_folder: Option<PathBuf>,
    pub set_file: Option<PathBuf>,
    pub file_select: bool,
    pub reset_session: bool,
    pub debug: bool,
    pub no_status_line: bool,
    pub poll_interval_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub journal: JournalConfig,
    pub monitor: MonitorConfig,
    pub log_levels: LogLevelConfig,
    pub matrix: Option<MatrixConfig>,
    pub web: WebConfig,
    pub config_source: ConfigSource,
    pub set_file: Option<PathBuf>,
    pub file_select: bool,
    pub reset_session: bool,
    pub debug: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedConfig {
    pub config: AppConfig,
    pub warnings: Vec<String>,
    pub source: ConfigSource,
}
