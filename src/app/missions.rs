use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::mission::{MissionKind, MissionProgress, MissionState, MissionTracker, TrackedMission};
use crate::text::line_safe;

use super::display::{credits_i64, display_timestamp, ValueDisplay};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MissionListView {
    pub active_count: usize,
    pub completed_count: usize,
    pub total_count: usize,
    pub status_label: String,
    pub items: Vec<MissionView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MissionView {
    pub mission_id: u64,
    pub state: String,
    pub state_label: String,
    pub kind: String,
    pub kind_label: String,
    pub display_name: String,
    pub issuing_faction: Option<String>,
    pub target_faction: Option<String>,
    pub destination_system: Option<String>,
    pub destination_station: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub accepted_at_display: String,
    pub expiry: Option<DateTime<Utc>>,
    pub expiry_display: Option<String>,
    pub reward: ValueDisplay<i64>,
    pub progress: MissionProgressView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MissionProgressView {
    None,
    Massacre {
        target: Option<String>,
        target_faction: Option<String>,
        kills: u64,
        kill_count: u64,
        display: String,
    },
    Trade {
        commodity: Option<String>,
        collected: u64,
        delivered: u64,
        count: u64,
        display: String,
    },
}

impl MissionListView {
    pub fn from_tracker(tracker: &MissionTracker) -> Self {
        let items: Vec<_> = tracker
            .missions()
            .values()
            .filter(|mission| is_active_mission_state(mission.state))
            .map(MissionView::from)
            .collect();
        let active_count = tracker
            .missions()
            .values()
            .filter(|mission| is_active_mission_state(mission.state))
            .count();
        let completed_count = tracker
            .missions()
            .values()
            .filter(|mission| mission.state == MissionState::Completed)
            .count();
        let total_count = tracker.missions().len();
        Self {
            active_count,
            completed_count,
            total_count,
            status_label: active_count.to_string(),
            items,
        }
    }
}

const fn is_active_mission_state(state: MissionState) -> bool {
    matches!(state, MissionState::Active | MissionState::Redirected)
}

impl From<&TrackedMission> for MissionView {
    fn from(mission: &TrackedMission) -> Self {
        Self {
            mission_id: mission.mission_id,
            state: mission_state_key(mission.state),
            state_label: mission_state_label(mission.state),
            kind: mission_kind_key(mission.kind),
            kind_label: mission_kind_label(mission.kind),
            display_name: line_safe(&mission_display_name(mission)),
            issuing_faction: option_line_safe(&mission.issuing_faction),
            target_faction: option_line_safe(&mission.target_faction),
            destination_system: option_line_safe(&mission.destination_system),
            destination_station: option_line_safe(&mission.destination_station),
            accepted_at: mission.accepted_at,
            accepted_at_display: display_timestamp(mission.accepted_at),
            expiry: mission.expiry,
            expiry_display: mission.expiry.map(display_timestamp),
            reward: credits_i64(mission.reward.unwrap_or(0)),
            progress: MissionProgressView::from(&mission.progress),
        }
    }
}

impl From<&MissionProgress> for MissionProgressView {
    fn from(progress: &MissionProgress) -> Self {
        match progress {
            MissionProgress::None => Self::None,
            MissionProgress::Massacre {
                target,
                target_faction,
                kill_count,
                kills,
                ..
            } => Self::Massacre {
                target: option_line_safe(target),
                target_faction: option_line_safe(target_faction),
                kills: *kills,
                kill_count: *kill_count,
                display: format!("{kills}/{kill_count} kills"),
            },
            MissionProgress::Trade {
                commodity,
                commodity_localised,
                count,
                items_collected,
                items_delivered,
            } => Self::Trade {
                commodity: commodity_localised
                    .as_deref()
                    .or(commodity.as_deref())
                    .map(line_safe),
                collected: *items_collected,
                delivered: *items_delivered,
                count: *count,
                display: format!("{items_delivered}/{count} delivered"),
            },
        }
    }
}

fn option_line_safe(value: &Option<String>) -> Option<String> {
    value.as_deref().map(line_safe)
}

fn mission_display_name(mission: &TrackedMission) -> String {
    mission
        .localised_name
        .clone()
        .or_else(|| mission.name.clone())
        .unwrap_or_else(|| format!("Mission {}", mission.mission_id))
}

fn mission_state_key(state: MissionState) -> String {
    match state {
        MissionState::Active => "active",
        MissionState::Redirected => "redirected",
        MissionState::Completed => "completed",
        MissionState::Failed => "failed",
        MissionState::Abandoned => "abandoned",
    }
    .to_string()
}

fn mission_state_label(state: MissionState) -> String {
    match state {
        MissionState::Active => "Active",
        MissionState::Redirected => "Redirected",
        MissionState::Completed => "Completed",
        MissionState::Failed => "Failed",
        MissionState::Abandoned => "Abandoned",
    }
    .to_string()
}

fn mission_kind_key(kind: MissionKind) -> String {
    match kind {
        MissionKind::Massacre => "massacre",
        MissionKind::Trade => "trade",
        MissionKind::Other => "other",
    }
    .to_string()
}

fn mission_kind_label(kind: MissionKind) -> String {
    match kind {
        MissionKind::Massacre => "Massacre",
        MissionKind::Trade => "Trade",
        MissionKind::Other => "Other",
    }
    .to_string()
}
