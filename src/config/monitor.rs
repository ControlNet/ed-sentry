use toml::Value;

use super::{read_bool, value_read};

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

pub(super) fn read_monitor_config(
    table: &toml::map::Map<String, Value>,
    monitor: &mut MonitorConfig,
    warnings: &mut Vec<String>,
) {
    read_bool(
        table.get("use_utc"),
        "monitor.use_utc",
        &mut monitor.use_utc,
        warnings,
    );
    read_bool(
        table.get("live_status"),
        "monitor.live_status",
        &mut monitor.live_status,
        warnings,
    );
    read_bool(
        table.get("dynamic_title"),
        "monitor.dynamic_title",
        &mut monitor.dynamic_title,
        warnings,
    );
    value_read::read_u16(
        table.get("warn_kill_rate"),
        "monitor.warn_kill_rate",
        &mut monitor.warn_kill_rate,
        warnings,
    );
    value_read::read_u16(
        table.get("warn_kill_rate_delay_minutes"),
        "monitor.warn_kill_rate_delay_minutes",
        &mut monitor.warn_kill_rate_delay_minutes,
        warnings,
    );
    value_read::read_u16(
        table.get("warn_no_kills_minutes"),
        "monitor.warn_no_kills_minutes",
        &mut monitor.warn_no_kills_minutes,
        warnings,
    );
    value_read::read_u16(
        table.get("warn_no_kills_initial_minutes"),
        "monitor.warn_no_kills_initial_minutes",
        &mut monitor.warn_no_kills_initial_minutes,
        warnings,
    );
    value_read::read_u16(
        table.get("warn_cooldown_minutes"),
        "monitor.warn_cooldown_minutes",
        &mut monitor.warn_cooldown_minutes,
        warnings,
    );
    value_read::read_u16(
        table.get("duplicate_max"),
        "monitor.duplicate_max",
        &mut monitor.duplicate_max,
        warnings,
    );
    read_bool(
        table.get("pirate_names"),
        "monitor.pirate_names",
        &mut monitor.pirate_names,
        warnings,
    );
    read_bool(
        table.get("bounty_faction"),
        "monitor.bounty_faction",
        &mut monitor.bounty_faction,
        warnings,
    );
    read_bool(
        table.get("bounty_value"),
        "monitor.bounty_value",
        &mut monitor.bounty_value,
        warnings,
    );
    read_bool(
        table.get("extended_stats"),
        "monitor.extended_stats",
        &mut monitor.extended_stats,
        warnings,
    );
    value_read::read_u8(
        table.get("min_scan_level"),
        "monitor.min_scan_level",
        &mut monitor.min_scan_level,
        warnings,
    );
    value_read::read_u64(
        table.get("poll_interval_ms"),
        "monitor.poll_interval_ms",
        &mut monitor.poll_interval_ms,
        warnings,
    );
}
