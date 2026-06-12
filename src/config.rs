use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use toml::Value;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AppConfig {
    pub journal: JournalConfig,
    pub monitor: MonitorConfig,
    pub log_levels: LogLevelConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalConfig {
    pub folder: String,
    pub recent_files: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorConfig {
    pub use_utc: bool,
    pub live_status: bool,
    pub dynamic_title: bool,
    pub warn_kill_rate: u16,
    pub warn_kill_rate_delay_minutes: u16,
    pub warn_no_kills_minutes: u16,
    pub warn_no_kills_initial_minutes: u16,
    pub warn_cooldown_minutes: u16,
    pub duplicate_max: u16,
    pub pirate_names: bool,
    pub bounty_faction: bool,
    pub bounty_value: bool,
    pub extended_stats: bool,
    pub min_scan_level: u8,
    pub poll_interval_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogLevelConfig {
    pub scan_incoming: u8,
    pub scan_easy: u8,
    pub scan_hard: u8,
    pub kill_easy: u8,
    pub kill_hard: u8,
    pub fighter_hull: u8,
    pub fighter_down: u8,
    pub ship_shields: u8,
    pub ship_hull: u8,
    pub died: u8,
    pub cargo_lost: u8,
    pub bait_value_low: u8,
    pub security_scan: u8,
    pub security_attack: u8,
    pub fuel_report: u8,
    pub fuel_low: u8,
    pub fuel_critical: u8,
    pub missions: u8,
    pub missions_all: u8,
    pub merits: u8,
    pub no_kills: u8,
    pub kill_rate: u8,
    pub summary_kills: u8,
    pub summary_faction: u8,
    pub summary_scans: u8,
    pub summary_bounties: u8,
    pub summary_merits: u8,
    pub duplicate_suppression: u8,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub journal: JournalConfig,
    pub monitor: MonitorConfig,
    pub log_levels: LogLevelConfig,
    pub set_file: Option<PathBuf>,
    pub file_select: bool,
    pub reset_session: bool,
    pub debug: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedConfig {
    pub config: AppConfig,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    MalformedToml {
        path: Option<PathBuf>,
        source: toml::de::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "failed to read config {}: {source}",
                    path.display()
                )
            }
            Self::MalformedToml { path, source } => match path {
                Some(path) => write!(
                    formatter,
                    "malformed TOML config {}: {source}",
                    path.display()
                ),
                None => write!(formatter, "malformed TOML config: {source}"),
            },
        }
    }
}

impl std::error::Error for ConfigError {}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            folder: String::new(),
            recent_files: 10,
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            use_utc: false,
            live_status: true,
            dynamic_title: true,
            warn_kill_rate: 20,
            warn_kill_rate_delay_minutes: 5,
            warn_no_kills_minutes: 20,
            warn_no_kills_initial_minutes: 5,
            warn_cooldown_minutes: 30,
            duplicate_max: 5,
            pirate_names: false,
            bounty_faction: false,
            bounty_value: false,
            extended_stats: false,
            min_scan_level: 1,
            poll_interval_ms: 1000,
        }
    }
}

impl Default for LogLevelConfig {
    fn default() -> Self {
        Self {
            scan_incoming: 1,
            scan_easy: 1,
            scan_hard: 2,
            kill_easy: 2,
            kill_hard: 2,
            fighter_hull: 2,
            fighter_down: 3,
            ship_shields: 3,
            ship_hull: 3,
            died: 3,
            cargo_lost: 3,
            bait_value_low: 2,
            security_scan: 2,
            security_attack: 3,
            fuel_report: 1,
            fuel_low: 2,
            fuel_critical: 3,
            missions: 2,
            missions_all: 3,
            merits: 0,
            no_kills: 3,
            kill_rate: 3,
            summary_kills: 2,
            summary_faction: 0,
            summary_scans: 0,
            summary_bounties: 2,
            summary_merits: 2,
            duplicate_suppression: 1,
        }
    }
}

impl AppConfig {
    pub fn from_toml_str(contents: &str) -> Result<LoadedConfig, ConfigError> {
        let value = contents
            .parse::<Value>()
            .map_err(|source| ConfigError::MalformedToml { path: None, source })?;
        Ok(Self::from_toml_value(&value))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<LoadedConfig, ConfigError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_toml_str(&contents).map_err(|error| match error {
            ConfigError::MalformedToml { source, .. } => ConfigError::MalformedToml {
                path: Some(path.to_path_buf()),
                source,
            },
            other => other,
        })
    }

    pub fn load_optional(path: Option<&Path>) -> Result<LoadedConfig, ConfigError> {
        match path {
            Some(path) => Self::load_from_path(path),
            None => Ok(LoadedConfig {
                config: Self::default(),
                warnings: Vec::new(),
            }),
        }
    }

    pub fn into_runtime(mut self, overrides: &CliConfigOverrides) -> RuntimeConfig {
        if let Some(folder) = &overrides.journal_folder {
            self.journal.folder = folder.to_string_lossy().into_owned();
        }
        if let Some(poll_interval_ms) = overrides.poll_interval_ms {
            self.monitor.poll_interval_ms = poll_interval_ms;
        }
        if overrides.no_status_line {
            self.monitor.live_status = false;
        }

        RuntimeConfig {
            journal: self.journal,
            monitor: self.monitor,
            log_levels: self.log_levels,
            set_file: overrides.set_file.clone(),
            file_select: overrides.file_select,
            reset_session: overrides.reset_session,
            debug: overrides.debug,
        }
    }

    fn from_toml_value(value: &Value) -> LoadedConfig {
        let mut config = Self::default();
        let mut warnings = Vec::new();

        if let Some(journal) = value.get("journal") {
            if let Some(table) = journal.as_table() {
                read_string(
                    table.get("folder"),
                    "journal.folder",
                    &mut config.journal.folder,
                    &mut warnings,
                );
                read_u16(
                    table.get("recent_files"),
                    "journal.recent_files",
                    &mut config.journal.recent_files,
                    &mut warnings,
                );
            } else {
                warnings.push(
                    "config key journal has wrong type; using defaults for section".to_string(),
                );
            }
        }

        if let Some(monitor) = value.get("monitor") {
            if let Some(table) = monitor.as_table() {
                read_bool(
                    table.get("use_utc"),
                    "monitor.use_utc",
                    &mut config.monitor.use_utc,
                    &mut warnings,
                );
                read_bool(
                    table.get("live_status"),
                    "monitor.live_status",
                    &mut config.monitor.live_status,
                    &mut warnings,
                );
                read_bool(
                    table.get("dynamic_title"),
                    "monitor.dynamic_title",
                    &mut config.monitor.dynamic_title,
                    &mut warnings,
                );
                read_u16(
                    table.get("warn_kill_rate"),
                    "monitor.warn_kill_rate",
                    &mut config.monitor.warn_kill_rate,
                    &mut warnings,
                );
                read_u16(
                    table.get("warn_kill_rate_delay_minutes"),
                    "monitor.warn_kill_rate_delay_minutes",
                    &mut config.monitor.warn_kill_rate_delay_minutes,
                    &mut warnings,
                );
                read_u16(
                    table.get("warn_no_kills_minutes"),
                    "monitor.warn_no_kills_minutes",
                    &mut config.monitor.warn_no_kills_minutes,
                    &mut warnings,
                );
                read_u16(
                    table.get("warn_no_kills_initial_minutes"),
                    "monitor.warn_no_kills_initial_minutes",
                    &mut config.monitor.warn_no_kills_initial_minutes,
                    &mut warnings,
                );
                read_u16(
                    table.get("warn_cooldown_minutes"),
                    "monitor.warn_cooldown_minutes",
                    &mut config.monitor.warn_cooldown_minutes,
                    &mut warnings,
                );
                read_u16(
                    table.get("duplicate_max"),
                    "monitor.duplicate_max",
                    &mut config.monitor.duplicate_max,
                    &mut warnings,
                );
                read_bool(
                    table.get("pirate_names"),
                    "monitor.pirate_names",
                    &mut config.monitor.pirate_names,
                    &mut warnings,
                );
                read_bool(
                    table.get("bounty_faction"),
                    "monitor.bounty_faction",
                    &mut config.monitor.bounty_faction,
                    &mut warnings,
                );
                read_bool(
                    table.get("bounty_value"),
                    "monitor.bounty_value",
                    &mut config.monitor.bounty_value,
                    &mut warnings,
                );
                read_bool(
                    table.get("extended_stats"),
                    "monitor.extended_stats",
                    &mut config.monitor.extended_stats,
                    &mut warnings,
                );
                read_u8(
                    table.get("min_scan_level"),
                    "monitor.min_scan_level",
                    &mut config.monitor.min_scan_level,
                    &mut warnings,
                );
                read_u64(
                    table.get("poll_interval_ms"),
                    "monitor.poll_interval_ms",
                    &mut config.monitor.poll_interval_ms,
                    &mut warnings,
                );
            } else {
                warnings.push(
                    "config key monitor has wrong type; using defaults for section".to_string(),
                );
            }
        }

        if let Some(log_levels) = value.get("log_levels") {
            if let Some(table) = log_levels.as_table() {
                read_log_levels(table, &mut config.log_levels, &mut warnings);
            } else {
                warnings.push(
                    "config key log_levels has wrong type; using defaults for section".to_string(),
                );
            }
        }

        LoadedConfig { config, warnings }
    }
}

fn read_log_levels(
    table: &toml::map::Map<String, Value>,
    log_levels: &mut LogLevelConfig,
    warnings: &mut Vec<String>,
) {
    read_u8(
        table.get("scan_incoming"),
        "log_levels.scan_incoming",
        &mut log_levels.scan_incoming,
        warnings,
    );
    read_u8(
        table.get("scan_easy"),
        "log_levels.scan_easy",
        &mut log_levels.scan_easy,
        warnings,
    );
    read_u8(
        table.get("scan_hard"),
        "log_levels.scan_hard",
        &mut log_levels.scan_hard,
        warnings,
    );
    read_u8(
        table.get("kill_easy"),
        "log_levels.kill_easy",
        &mut log_levels.kill_easy,
        warnings,
    );
    read_u8(
        table.get("kill_hard"),
        "log_levels.kill_hard",
        &mut log_levels.kill_hard,
        warnings,
    );
    read_u8(
        table.get("fighter_hull"),
        "log_levels.fighter_hull",
        &mut log_levels.fighter_hull,
        warnings,
    );
    read_u8(
        table.get("fighter_down"),
        "log_levels.fighter_down",
        &mut log_levels.fighter_down,
        warnings,
    );
    read_u8(
        table.get("ship_shields"),
        "log_levels.ship_shields",
        &mut log_levels.ship_shields,
        warnings,
    );
    read_u8(
        table.get("ship_hull"),
        "log_levels.ship_hull",
        &mut log_levels.ship_hull,
        warnings,
    );
    read_u8(
        table.get("died"),
        "log_levels.died",
        &mut log_levels.died,
        warnings,
    );
    read_u8(
        table.get("cargo_lost"),
        "log_levels.cargo_lost",
        &mut log_levels.cargo_lost,
        warnings,
    );
    read_u8(
        table.get("bait_value_low"),
        "log_levels.bait_value_low",
        &mut log_levels.bait_value_low,
        warnings,
    );
    read_u8(
        table.get("security_scan"),
        "log_levels.security_scan",
        &mut log_levels.security_scan,
        warnings,
    );
    read_u8(
        table.get("security_attack"),
        "log_levels.security_attack",
        &mut log_levels.security_attack,
        warnings,
    );
    read_u8(
        table.get("fuel_report"),
        "log_levels.fuel_report",
        &mut log_levels.fuel_report,
        warnings,
    );
    read_u8(
        table.get("fuel_low"),
        "log_levels.fuel_low",
        &mut log_levels.fuel_low,
        warnings,
    );
    read_u8(
        table.get("fuel_critical"),
        "log_levels.fuel_critical",
        &mut log_levels.fuel_critical,
        warnings,
    );
    read_u8(
        table.get("missions"),
        "log_levels.missions",
        &mut log_levels.missions,
        warnings,
    );
    read_u8(
        table.get("missions_all"),
        "log_levels.missions_all",
        &mut log_levels.missions_all,
        warnings,
    );
    read_u8(
        table.get("merits"),
        "log_levels.merits",
        &mut log_levels.merits,
        warnings,
    );
    read_u8(
        table.get("no_kills"),
        "log_levels.no_kills",
        &mut log_levels.no_kills,
        warnings,
    );
    read_u8(
        table.get("kill_rate"),
        "log_levels.kill_rate",
        &mut log_levels.kill_rate,
        warnings,
    );
    read_u8(
        table.get("summary_kills"),
        "log_levels.summary_kills",
        &mut log_levels.summary_kills,
        warnings,
    );
    read_u8(
        table.get("summary_faction"),
        "log_levels.summary_faction",
        &mut log_levels.summary_faction,
        warnings,
    );
    read_u8(
        table.get("summary_scans"),
        "log_levels.summary_scans",
        &mut log_levels.summary_scans,
        warnings,
    );
    read_u8(
        table.get("summary_bounties"),
        "log_levels.summary_bounties",
        &mut log_levels.summary_bounties,
        warnings,
    );
    read_u8(
        table.get("summary_merits"),
        "log_levels.summary_merits",
        &mut log_levels.summary_merits,
        warnings,
    );
    read_u8(
        table.get("duplicate_suppression"),
        "log_levels.duplicate_suppression",
        &mut log_levels.duplicate_suppression,
        warnings,
    );
}

fn read_string(value: Option<&Value>, key: &str, target: &mut String, warnings: &mut Vec<String>) {
    if let Some(value) = value {
        if let Some(parsed) = value.as_str() {
            *target = parsed.to_string();
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

fn read_bool(value: Option<&Value>, key: &str, target: &mut bool, warnings: &mut Vec<String>) {
    if let Some(value) = value {
        if let Some(parsed) = value.as_bool() {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

fn read_u8(value: Option<&Value>, key: &str, target: &mut u8, warnings: &mut Vec<String>) {
    if let Some(value) = value {
        if let Some(parsed) = value
            .as_integer()
            .and_then(|number| u8::try_from(number).ok())
        {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

fn read_u16(value: Option<&Value>, key: &str, target: &mut u16, warnings: &mut Vec<String>) {
    if let Some(value) = value {
        if let Some(parsed) = value
            .as_integer()
            .and_then(|number| u16::try_from(number).ok())
        {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

fn read_u64(value: Option<&Value>, key: &str, target: &mut u64, warnings: &mut Vec<String>) {
    if let Some(value) = value {
        if let Some(parsed) = value
            .as_integer()
            .and_then(|number| u64::try_from(number).ok())
        {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

fn wrong_type_warning(key: &str) -> String {
    format!("config key {key} has wrong type; using default")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_config_defaults_match_locked_contract() {
        let config = AppConfig::default();

        assert_eq!(config.journal.folder, "");
        assert_eq!(config.journal.recent_files, 10);
        assert!(!config.monitor.use_utc);
        assert!(config.monitor.live_status);
        assert!(config.monitor.dynamic_title);
        assert_eq!(config.monitor.warn_kill_rate, 20);
        assert_eq!(config.monitor.warn_kill_rate_delay_minutes, 5);
        assert_eq!(config.monitor.warn_no_kills_minutes, 20);
        assert_eq!(config.monitor.warn_no_kills_initial_minutes, 5);
        assert_eq!(config.monitor.warn_cooldown_minutes, 30);
        assert_eq!(config.monitor.duplicate_max, 5);
        assert!(!config.monitor.pirate_names);
        assert!(!config.monitor.bounty_faction);
        assert!(!config.monitor.bounty_value);
        assert!(!config.monitor.extended_stats);
        assert_eq!(config.monitor.min_scan_level, 1);
        assert_eq!(config.monitor.poll_interval_ms, 1000);
        assert_eq!(config.log_levels.summary_faction, 0);
        assert_eq!(config.log_levels.summary_scans, 0);
        assert_eq!(config.log_levels.merits, 0);
        assert_eq!(config.log_levels.summary_merits, 2);
        assert_eq!(config.log_levels.duplicate_suppression, 1);
    }

    #[test]
    fn cli_config_toml_values_override_defaults() {
        let loaded = AppConfig::from_toml_str(
            r#"
            [journal]
            folder = "/journals"

            [monitor]
            live_status = false
            poll_interval_ms = 2500

            [log_levels]
            duplicate_suppression = 2
            "#,
        )
        .unwrap();

        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.config.journal.folder, "/journals");
        assert_eq!(loaded.config.journal.recent_files, 10);
        assert!(!loaded.config.monitor.live_status);
        assert_eq!(loaded.config.monitor.poll_interval_ms, 2500);
        assert_eq!(loaded.config.log_levels.duplicate_suppression, 2);
    }

    #[test]
    fn cli_config_wrong_typed_keys_warn_and_keep_defaults() {
        let loaded = AppConfig::from_toml_str(
            r#"
            [monitor]
            poll_interval_ms = "fast"

            [log_levels]
            duplicate_suppression = "loud"
            "#,
        )
        .unwrap();

        assert_eq!(loaded.config.monitor.poll_interval_ms, 1000);
        assert_eq!(loaded.config.log_levels.duplicate_suppression, 1);
        assert_eq!(loaded.warnings.len(), 2);
        assert!(loaded.warnings[0].contains("monitor.poll_interval_ms"));
        assert!(loaded.warnings[1].contains("log_levels.duplicate_suppression"));
    }

    #[test]
    fn cli_config_cli_overrides_toml_config() {
        let loaded = AppConfig::from_toml_str(
            r#"
            [journal]
            folder = "/toml/journals"

            [monitor]
            live_status = true
            poll_interval_ms = 1500
            "#,
        )
        .unwrap();

        let runtime = loaded.config.into_runtime(&CliConfigOverrides {
            journal_folder: Some(PathBuf::from("/cli/journals")),
            set_file: Some(PathBuf::from("Journal.log")),
            no_status_line: true,
            poll_interval_ms: Some(500),
            debug: true,
            ..CliConfigOverrides::default()
        });

        assert_eq!(runtime.journal.folder, "/cli/journals");
        assert_eq!(runtime.monitor.poll_interval_ms, 500);
        assert!(!runtime.monitor.live_status);
        assert_eq!(runtime.set_file, Some(PathBuf::from("Journal.log")));
        assert!(runtime.debug);
    }
}
