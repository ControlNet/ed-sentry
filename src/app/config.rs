use crate::config::{
    JournalConfig, LogLevelConfig, MatrixConfig, MonitorConfig, RuntimeConfig, WebConfig,
};

mod types;

pub use types::{
    ConfigApiView, ConfigEndpointPolicy, EditableConfigUpdate, EditableConfigView,
    JournalConfigEdit, JournalConfigView, LogLevelConfigEdit, LogLevelConfigView, MatrixConfigEdit,
    MatrixConfigView, MonitorConfigEdit, MonitorConfigView, WebConfigEdit, WebConfigView,
};

impl EditableConfigView {
    pub fn from_runtime_config(config: &RuntimeConfig) -> Self {
        Self {
            journal: JournalConfigView::from(&config.journal),
            monitor: MonitorConfigView::from(&config.monitor),
            log_levels: LogLevelConfigView::from(&config.log_levels),
            matrix: config.matrix.as_ref().map(MatrixConfigView::from),
            web: WebConfigView::from(&config.web),
        }
    }
}

impl From<&JournalConfig> for JournalConfigView {
    fn from(config: &JournalConfig) -> Self {
        Self {
            folder: config.folder.clone(),
            recent_files: config.recent_files,
        }
    }
}

impl From<&MonitorConfig> for MonitorConfigView {
    fn from(config: &MonitorConfig) -> Self {
        Self {
            use_utc: config.use_utc,
            live_status: config.live_status,
            dynamic_title: config.dynamic_title,
            warn_kill_rate: config.warn_kill_rate,
            warn_kill_rate_delay_minutes: config.warn_kill_rate_delay_minutes,
            warn_no_kills_minutes: config.warn_no_kills_minutes,
            warn_no_kills_initial_minutes: config.warn_no_kills_initial_minutes,
            warn_cooldown_minutes: config.warn_cooldown_minutes,
            duplicate_max: config.duplicate_max,
            pirate_names: config.pirate_names,
            bounty_faction: config.bounty_faction,
            bounty_value: config.bounty_value,
            extended_stats: config.extended_stats,
            min_scan_level: config.min_scan_level,
            poll_interval_ms: config.poll_interval_ms,
        }
    }
}

impl From<&LogLevelConfig> for LogLevelConfigView {
    fn from(config: &LogLevelConfig) -> Self {
        Self {
            scan_incoming: config.scan_incoming,
            scan_easy: config.scan_easy,
            scan_hard: config.scan_hard,
            kill_easy: config.kill_easy,
            kill_hard: config.kill_hard,
            fighter_hull: config.fighter_hull,
            fighter_down: config.fighter_down,
            ship_shields: config.ship_shields,
            ship_hull: config.ship_hull,
            died: config.died,
            cargo_lost: config.cargo_lost,
            bait_value_low: config.bait_value_low,
            security_scan: config.security_scan,
            security_attack: config.security_attack,
            fuel_report: config.fuel_report,
            fuel_low: config.fuel_low,
            fuel_critical: config.fuel_critical,
            missions: config.missions,
            missions_all: config.missions_all,
            merits: config.merits,
            rank_promotion: config.rank_promotion,
            no_kills: config.no_kills,
            kill_rate: config.kill_rate,
            summary_kills: config.summary_kills,
            summary_faction: config.summary_faction,
            summary_scans: config.summary_scans,
            summary_bounties: config.summary_bounties,
            summary_merits: config.summary_merits,
            duplicate_suppression: config.duplicate_suppression,
        }
    }
}

impl From<&MatrixConfig> for MatrixConfigView {
    fn from(config: &MatrixConfig) -> Self {
        Self {
            enabled: config.enabled,
            homeserver: config.homeserver.clone(),
            room_id: config.room_id.clone(),
            mention_user_id: config.mention_user_id.clone(),
            status_update_interval_seconds: config.status_update_interval_seconds,
            access_token_present: config.access_token.is_some(),
            access_token_replacement: None,
        }
    }
}

impl From<&WebConfig> for WebConfigView {
    fn from(config: &WebConfig) -> Self {
        Self {
            enabled: config.enabled,
            host: config.host.clone(),
            port: config.port,
            open_browser: config.open_browser,
            status_label: if config.enabled {
                "Enabled"
            } else {
                "Disabled"
            }
            .to_string(),
        }
    }
}
