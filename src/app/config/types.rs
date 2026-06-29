use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConfigApiView {
    pub version: u8,
    pub config: EditableConfigView,
    pub policy: ConfigEndpointPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConfigEndpointPolicy {
    pub state_changing_enabled: bool,
    pub state_changing_reason: String,
    pub remote_bind: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EditableConfigView {
    pub journal: JournalConfigView,
    pub monitor: MonitorConfigView,
    pub log_levels: LogLevelConfigView,
    pub matrix: Option<MatrixConfigView>,
    pub web: WebConfigView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct JournalConfigView {
    pub folder: String,
    pub recent_files: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct JournalConfigEdit {
    pub folder: Option<String>,
    pub recent_files: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MonitorConfigView {
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MonitorConfigEdit {
    pub use_utc: Option<bool>,
    pub live_status: Option<bool>,
    pub dynamic_title: Option<bool>,
    pub warn_kill_rate: Option<u16>,
    pub warn_kill_rate_delay_minutes: Option<u16>,
    pub warn_no_kills_minutes: Option<u16>,
    pub warn_no_kills_initial_minutes: Option<u16>,
    pub warn_cooldown_minutes: Option<u16>,
    pub duplicate_max: Option<u16>,
    pub pirate_names: Option<bool>,
    pub bounty_faction: Option<bool>,
    pub bounty_value: Option<bool>,
    pub extended_stats: Option<bool>,
    pub min_scan_level: Option<u8>,
    pub poll_interval_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LogLevelConfigView {
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
    pub rank_promotion: u8,
    pub no_kills: u8,
    pub kill_rate: u8,
    pub summary_kills: u8,
    pub summary_faction: u8,
    pub summary_scans: u8,
    pub summary_bounties: u8,
    pub summary_merits: u8,
    pub duplicate_suppression: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct LogLevelConfigEdit {
    pub scan_incoming: Option<u8>,
    pub scan_easy: Option<u8>,
    pub scan_hard: Option<u8>,
    pub kill_easy: Option<u8>,
    pub kill_hard: Option<u8>,
    pub fighter_hull: Option<u8>,
    pub fighter_down: Option<u8>,
    pub ship_shields: Option<u8>,
    pub ship_hull: Option<u8>,
    pub died: Option<u8>,
    pub cargo_lost: Option<u8>,
    pub bait_value_low: Option<u8>,
    pub security_scan: Option<u8>,
    pub security_attack: Option<u8>,
    pub fuel_report: Option<u8>,
    pub fuel_low: Option<u8>,
    pub fuel_critical: Option<u8>,
    pub missions: Option<u8>,
    pub missions_all: Option<u8>,
    pub merits: Option<u8>,
    pub rank_promotion: Option<u8>,
    pub no_kills: Option<u8>,
    pub kill_rate: Option<u8>,
    pub summary_kills: Option<u8>,
    pub summary_faction: Option<u8>,
    pub summary_scans: Option<u8>,
    pub summary_bounties: Option<u8>,
    pub summary_merits: Option<u8>,
    pub duplicate_suppression: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MatrixConfigView {
    pub enabled: bool,
    pub homeserver: Option<String>,
    pub room_id: Option<String>,
    pub mention_user_id: Option<String>,
    pub status_update_interval_seconds: u64,
    pub access_token_present: bool,
    #[serde(default, skip_serializing)]
    pub access_token_replacement: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct MatrixConfigEdit {
    pub enabled: bool,
    pub homeserver: Option<String>,
    pub room_id: Option<String>,
    pub mention_user_id: Option<String>,
    pub status_update_interval_seconds: u64,
    pub access_token_replacement: Option<String>,
    pub clear_access_token: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WebConfigView {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub open_browser: bool,
    pub status_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct WebConfigEdit {
    pub enabled: Option<bool>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub open_browser: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct EditableConfigUpdate {
    pub journal: Option<JournalConfigEdit>,
    pub monitor: Option<MonitorConfigEdit>,
    pub log_levels: Option<LogLevelConfigEdit>,
    pub matrix: Option<MatrixConfigEdit>,
    pub web: Option<WebConfigEdit>,
}
