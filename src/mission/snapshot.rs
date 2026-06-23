use chrono::{DateTime, TimeDelta, Utc};

use crate::event::{MissionListItem, MissionsEvent};

use super::{empty_progress, mission_kind_from_name, MissionState, MissionTracker, TrackedMission};

impl MissionTracker {
    pub(in crate::mission) fn apply_missions_snapshot(&mut self, event: &MissionsEvent) {
        let active_ids = event.active.iter().filter_map(|mission| mission.mission_id);
        self.retain_snapshot(active_ids);
        for mission in &event.active {
            self.apply_active_snapshot_mission(event, mission);
        }
    }

    fn apply_active_snapshot_mission(&mut self, event: &MissionsEvent, item: &MissionListItem) {
        let Some(mission_id) = item.mission_id else {
            return;
        };
        if self.missions.contains_key(&mission_id) {
            return;
        }

        let kind = mission_kind_from_name(item.name.as_deref());
        self.missions.insert(
            mission_id,
            TrackedMission {
                mission_id,
                state: MissionState::Active,
                kind,
                name: item.name.clone(),
                localised_name: None,
                issuing_faction: None,
                target_faction: None,
                accepted_at: event.timestamp,
                completion_time: None,
                expiry: snapshot_expiry(event.timestamp, item.expires),
                reward: None,
                influence: None,
                reputation: None,
                wing: None,
                origin: self.origin.clone(),
                destination_system: None,
                destination_station: None,
                destination_settlement: None,
                progress: empty_progress(kind),
            },
        );
    }
}

fn snapshot_expiry(timestamp: DateTime<Utc>, expires: Option<u64>) -> Option<DateTime<Utc>> {
    let seconds = i64::try_from(expires?).ok()?;
    if seconds == 0 {
        return None;
    }

    timestamp.checked_add_signed(TimeDelta::seconds(seconds))
}
