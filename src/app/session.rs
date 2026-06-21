use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use crate::state::SessionState;
use crate::time::format_duration;

use super::display::{credits_u64, display_timestamp, RateView, ValueDisplay};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SessionView {
    pub commander: Option<String>,
    pub ship: Option<String>,
    pub system: Option<String>,
    pub mode: Option<String>,
    pub active: bool,
    pub status_label: String,
    pub started_at: Option<DateTime<Utc>>,
    pub started_at_display: Option<String>,
    pub ended_at: Option<DateTime<Utc>>,
    pub ended_at_display: Option<String>,
    pub elapsed_seconds: i64,
    pub elapsed_display: String,
    pub shields_up: Option<bool>,
    pub shields_display: String,
    pub ship_hull_percent: Option<f64>,
    pub ship_hull_display: String,
    pub fighter_hull_percent: Option<f64>,
    pub fighter_hull_display: String,
    pub fighter_alive: Option<bool>,
    pub kills: u64,
    pub scans: u64,
    pub bounty_total: ValueDisplay<u64>,
    pub merits: u64,
    pub merits_to_report: u64,
    pub kill_total_rate_per_hour: RateView,
    pub kill_recent_rate_per_hour: RateView,
    pub scan_total_rate_per_hour: RateView,
    pub scan_recent_rate_per_hour: RateView,
    pub last_kill_at: Option<DateTime<Utc>>,
    pub last_kill_display: Option<String>,
    pub last_scan_at: Option<DateTime<Utc>>,
    pub last_scan_display: Option<String>,
}

impl SessionView {
    pub fn from_state(state: &SessionState, now: DateTime<Utc>) -> Self {
        let elapsed = elapsed_duration(state, now);
        Self {
            commander: state.commander.clone(),
            ship: state.ship.clone(),
            system: state.system.clone(),
            mode: state.mode.clone(),
            active: state.active_session,
            status_label: session_status_label(state),
            started_at: state.session_started_at,
            started_at_display: state.session_started_at.map(display_timestamp),
            ended_at: state.session_ended_at,
            ended_at_display: state.session_ended_at.map(display_timestamp),
            elapsed_seconds: elapsed.num_seconds().max(0),
            elapsed_display: format_duration(elapsed),
            shields_up: state.shields_up,
            shields_display: shields_display(state.shields_up),
            ship_hull_percent: state.ship_hull,
            ship_hull_display: percent_display(state.ship_hull),
            fighter_hull_percent: state.fighter_hull,
            fighter_hull_display: percent_display(state.fighter_hull),
            fighter_alive: state.fighter_alive,
            kills: state.kills,
            scans: state.cargo_scans,
            bounty_total: credits_u64(state.bounty_total),
            merits: state.merits,
            merits_to_report: state.merits_to_report,
            kill_total_rate_per_hour: RateView::new(state.total_kill_rate_per_hour_at(now)),
            kill_recent_rate_per_hour: RateView::new(state.recent_kill_rate_per_hour_at(now)),
            scan_total_rate_per_hour: RateView::new(state.total_scan_rate_per_hour_at(now)),
            scan_recent_rate_per_hour: RateView::new(state.recent_scan_rate_per_hour_at(now)),
            last_kill_at: state.last_kill_at,
            last_kill_display: state.last_kill_at.map(display_timestamp),
            last_scan_at: state.last_scan_at,
            last_scan_display: state.last_scan_at.map(display_timestamp),
        }
    }
}

fn elapsed_duration(state: &SessionState, now: DateTime<Utc>) -> Duration {
    let Some(started_at) = state.session_started_at else {
        return Duration::zero();
    };
    state
        .session_ended_at
        .unwrap_or(now)
        .signed_duration_since(started_at)
}

fn session_status_label(state: &SessionState) -> String {
    if state.active_session {
        return "Active".to_string();
    }
    if state.session_ended_at.is_some() {
        return "Ended".to_string();
    }
    "Idle".to_string()
}

fn shields_display(shields_up: Option<bool>) -> String {
    match shields_up {
        Some(true) => "Up",
        Some(false) => "Down",
        None => "Unknown",
    }
    .to_string()
}

fn percent_display(value: Option<f64>) -> String {
    value
        .map(|value| format!("{:.0}%", (value * 100.0).clamp(0.0, 100.0)))
        .unwrap_or_else(|| "Unknown".to_string())
}
