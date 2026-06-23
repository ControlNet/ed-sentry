use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::event::MissionEvent;

mod snapshot;
mod tracker;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissionState {
    Active,
    Redirected,
    Completed,
    Failed,
    Abandoned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissionKind {
    Massacre,
    Trade,
    Other,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MissionOrigin {
    pub system_address: Option<i64>,
    pub system_name: Option<String>,
    pub market_id: Option<u64>,
    pub station_name: Option<String>,
    pub odyssey: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MissionProgress {
    None,
    Massacre {
        target: Option<String>,
        target_type: Option<String>,
        target_faction: Option<String>,
        target_system: Option<String>,
        kill_count: u64,
        kills: u64,
    },
    Trade {
        commodity: Option<String>,
        commodity_localised: Option<String>,
        count: u64,
        items_collected: u64,
        items_delivered: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackedMission {
    pub mission_id: u64,
    pub state: MissionState,
    pub kind: MissionKind,
    pub name: Option<String>,
    pub localised_name: Option<String>,
    pub issuing_faction: Option<String>,
    pub target_faction: Option<String>,
    pub accepted_at: DateTime<Utc>,
    pub completion_time: Option<DateTime<Utc>>,
    pub expiry: Option<DateTime<Utc>>,
    pub reward: Option<i64>,
    pub influence: Option<String>,
    pub reputation: Option<String>,
    pub wing: Option<bool>,
    pub origin: MissionOrigin,
    pub destination_system: Option<String>,
    pub destination_station: Option<String>,
    pub destination_settlement: Option<String>,
    pub progress: MissionProgress,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MissionTracker {
    missions: BTreeMap<u64, TrackedMission>,
    origin: MissionOrigin,
}

fn completion_reward(event: &MissionEvent) -> Option<i64> {
    match (event.reward, event.donated) {
        (Some(0), Some(donated)) => Some(-donated),
        (Some(reward), _) => Some(reward),
        (None, Some(donated)) => Some(-donated),
        (None, None) => None,
    }
}

fn invalid_massacre_victim(victim_faction: &str) -> bool {
    let victim = victim_faction.to_ascii_lowercase();
    victim.contains("faction_none") || victim.contains("faction_pirate")
}

fn mission_kind(event: &MissionEvent) -> MissionKind {
    mission_kind_from_values(event.kill_count.unwrap_or(0), &mission_text(event))
}

fn mission_kind_from_name(name: Option<&str>) -> MissionKind {
    mission_kind_from_values(0, &name.unwrap_or_default().to_ascii_lowercase())
}

fn mission_kind_from_values(kill_count: u64, text: &str) -> MissionKind {
    if kill_count > 0 || text.contains("massacre") {
        return MissionKind::Massacre;
    }
    if [
        "mission_collect",
        "mission_delivery",
        "mission_mining",
        "mission_altruism",
    ]
    .iter()
    .any(|needle| text.contains(needle))
    {
        return MissionKind::Trade;
    }
    MissionKind::Other
}

fn mission_progress(event: &MissionEvent) -> MissionProgress {
    match mission_kind(event) {
        MissionKind::Massacre => MissionProgress::Massacre {
            target: event.target.clone(),
            target_type: event.target_type.clone(),
            target_faction: event.target_faction.clone(),
            target_system: event.destination_system.clone(),
            kill_count: event.kill_count.unwrap_or(0),
            kills: 0,
        },
        MissionKind::Trade => MissionProgress::Trade {
            commodity: event.commodity.clone(),
            commodity_localised: event.commodity_localised.clone(),
            count: event.count.unwrap_or(0),
            items_collected: 0,
            items_delivered: 0,
        },
        MissionKind::Other => MissionProgress::None,
    }
}

fn empty_progress(kind: MissionKind) -> MissionProgress {
    match kind {
        MissionKind::Massacre => MissionProgress::Massacre {
            target: None,
            target_type: None,
            target_faction: None,
            target_system: None,
            kill_count: 0,
            kills: 0,
        },
        MissionKind::Trade => MissionProgress::Trade {
            commodity: None,
            commodity_localised: None,
            count: 0,
            items_collected: 0,
            items_delivered: 0,
        },
        MissionKind::Other => MissionProgress::None,
    }
}

fn mission_text(event: &MissionEvent) -> String {
    format!(
        "{} {} {}",
        event.name.as_deref().unwrap_or_default(),
        event.localised_name.as_deref().unwrap_or_default(),
        event.target_type.as_deref().unwrap_or_default()
    )
    .to_ascii_lowercase()
}
