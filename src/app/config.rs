use crate::config::{
    JournalConfig, LogLevelConfig, MatrixConfig, MonitorConfig, RuntimeConfig, TunnelConfig,
    WebConfig,
};

mod types;

pub use types::{
    ConfigApiView, ConfigEndpointPolicy, EditableConfigUpdate, EditableConfigView,
    JournalConfigEdit, JournalConfigView, LogLevelConfigEdit, LogLevelConfigView, MatrixConfigEdit,
    MatrixConfigView, MonitorConfigEdit, MonitorConfigView, TunnelConfigEdit, TunnelConfigView,
    WebConfigEdit, WebConfigView,
};

impl EditableConfigView {
    pub fn from_runtime_config(config: &RuntimeConfig) -> Self {
        Self {
            journal: JournalConfigView::from(&config.journal),
            monitor: MonitorConfigView::from(&config.monitor),
            log_levels: LogLevelConfigView::from(&config.log_levels),
            matrix: config.matrix.as_ref().map(MatrixConfigView::from),
            tunnel: TunnelConfigView::from(&config.tunnel),
            web: WebConfigView::from(&config.web),
        }
    }
}

pub fn restart_required_sections(
    running: &RuntimeConfig,
    updated: &RuntimeConfig,
    tunnel_running: bool,
) -> Vec<String> {
    let mut sections = Vec::new();
    if running.journal != updated.journal {
        sections.push("Journal".to_string());
    }
    if matrix_restart_required(running.matrix.as_ref(), updated.matrix.as_ref()) {
        sections.push("Matrix".to_string());
    }
    if running.web != updated.web {
        sections.push("Web UI".to_string());
    }
    if running.tunnel.auto_start != updated.tunnel.auto_start
        || (tunnel_running && running.tunnel.provider != updated.tunnel.provider)
    {
        sections.push("Tunnel".to_string());
    }
    sections
}

fn matrix_restart_required(running: Option<&MatrixConfig>, updated: Option<&MatrixConfig>) -> bool {
    let running_enabled = running.is_some_and(|matrix| matrix.enabled);
    let updated_enabled = updated.is_some_and(|matrix| matrix.enabled);
    if running_enabled != updated_enabled {
        return true;
    }
    running_enabled && running != updated
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

impl From<&TunnelConfig> for TunnelConfigView {
    fn from(config: &TunnelConfig) -> Self {
        Self {
            provider: config.provider.clone(),
            auto_start: config.auto_start,
            config_password_present: !config.config_password.is_empty(),
            config_password_replacement: None,
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

#[cfg(test)]
mod tests {
    use crate::config::{RuntimeConfig, TunnelConfig};

    use super::EditableConfigView;

    #[test]
    fn tunnel_config_api_view_redacts_existing_password_when_present() {
        // Given: a runtime config with a non-empty local tunnel config password.
        let config = RuntimeConfig {
            tunnel: TunnelConfig {
                provider: "cloudflare_quick".to_string(),
                auto_start: true,
                config_password: "fixture-tunnel-password".to_string(),
            },
            ..RuntimeConfig::default()
        };

        // When: the editable config API view is serialized for frontend clients.
        let view = EditableConfigView::from_runtime_config(&config);
        let json = serde_json::to_string(&view).unwrap();

        // Then: only password presence is exposed; plaintext and replacement fields stay local.
        assert!(view.tunnel.config_password_present);
        assert!(json.contains("config_password_present"), "{json}");
        assert!(!json.contains("fixture-tunnel-password"), "{json}");
        assert!(!json.contains("config_password_replacement"), "{json}");
    }

    #[test]
    fn restart_required_sections_distinguishes_live_and_restart_only_fields() {
        let running = RuntimeConfig::default();

        let mut live_updated = running.clone();
        live_updated.monitor.use_utc = !running.monitor.use_utc;
        live_updated.monitor.live_status = !running.monitor.live_status;
        live_updated.monitor.dynamic_title = !running.monitor.dynamic_title;
        live_updated.monitor.warn_kill_rate = 40;
        live_updated.monitor.poll_interval_ms = 200;
        live_updated.log_levels.scan_easy = 0;
        live_updated.tunnel.config_password = "updated-password".to_string();
        live_updated.tunnel.provider = "ssh".to_string();
        assert!(super::restart_required_sections(&running, &live_updated, false).is_empty());

        assert_eq!(
            super::restart_required_sections(&running, &live_updated, true),
            vec!["Tunnel"]
        );

        let mut recent_files_updated = running.clone();
        recent_files_updated.journal.recent_files = running.journal.recent_files + 5;
        assert_eq!(
            super::restart_required_sections(&running, &recent_files_updated, false),
            vec!["Journal"]
        );

        let mut auto_start_updated = running.clone();
        auto_start_updated.tunnel.auto_start = !running.tunnel.auto_start;
        assert_eq!(
            super::restart_required_sections(&running, &auto_start_updated, false),
            vec!["Tunnel"]
        );

        let mut disabled_matrix_updated = running.clone();
        disabled_matrix_updated.matrix = Some(crate::config::MatrixConfig {
            enabled: false,
            homeserver: Some("https://matrix.invalid".to_string()),
            ..crate::config::MatrixConfig::default()
        });
        assert!(
            super::restart_required_sections(&running, &disabled_matrix_updated, false).is_empty()
        );

        let mut enabled_matrix_updated = disabled_matrix_updated.clone();
        enabled_matrix_updated.matrix.as_mut().unwrap().enabled = true;
        assert_eq!(
            super::restart_required_sections(&running, &enabled_matrix_updated, false),
            vec!["Matrix"]
        );
    }
}
