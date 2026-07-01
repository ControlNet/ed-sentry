use std::collections::{BTreeMap, BTreeSet};

use crate::event::{
    BountyEvent, CargoDepotEvent, DockedEvent, JournalEvent, LocationEvent, MissionEvent,
    TravelEvent,
};

use super::{
    completion_reward, invalid_massacre_victim, mission_kind, mission_progress, MissionProgress,
    MissionState, MissionTracker, TrackedMission,
};

impl MissionTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn missions(&self) -> &BTreeMap<u64, TrackedMission> {
        &self.missions
    }

    pub fn mission(&self, mission_id: u64) -> Option<&TrackedMission> {
        self.missions.get(&mission_id)
    }

    pub fn apply_event(&mut self, event: &JournalEvent) {
        match event {
            JournalEvent::LoadGame(event) => self.origin.odyssey = event.odyssey,
            JournalEvent::Location(event) => self.apply_location(event),
            JournalEvent::Docked(event) => self.apply_docked(event),
            JournalEvent::Undocked(_) => self.clear_station_origin(),
            JournalEvent::FSDJump(event) => self.apply_travel(event),
            JournalEvent::MissionAccepted(event) => self.apply_accepted(event),
            JournalEvent::MissionRedirected(event) => {
                self.update_mission(event, MissionState::Redirected)
            }
            JournalEvent::MissionCompleted(event) => {
                self.update_mission(event, MissionState::Completed)
            }
            JournalEvent::MissionFailed(event) => self.update_mission(event, MissionState::Failed),
            JournalEvent::MissionAbandoned(event) => {
                self.update_mission(event, MissionState::Abandoned)
            }
            JournalEvent::Missions(event) => {
                if event.active_present {
                    self.apply_missions_snapshot(event);
                }
            }
            JournalEvent::CargoDepot(event) => self.apply_cargo_depot(event),
            JournalEvent::Bounty(event) => self.apply_bounty(event),
            JournalEvent::FactionKillBond(event) if event.reward.unwrap_or(0) > 0 => {
                if let Some(victim_faction) = event.victim_faction.as_deref() {
                    self.apply_massacre_kill(victim_faction);
                }
            }
            _ => {}
        }
    }

    fn apply_location(&mut self, event: &LocationEvent) {
        self.origin.system_name = event.star_system.clone();
        self.origin.system_address = event.system_address;
        if event.docked == Some(true)
            || event.station_name.is_some()
            || event.station_name_localised.is_some()
        {
            self.origin.station_name = event
                .station_name_localised
                .clone()
                .or_else(|| event.station_name.clone());
            self.origin.market_id = event.market_id;
        } else if event.docked == Some(false) {
            self.clear_station_origin();
        }
    }

    fn apply_docked(&mut self, event: &DockedEvent) {
        self.origin.system_name = event.star_system.clone();
        self.origin.system_address = event.system_address;
        self.origin.station_name = event
            .station_name_localised
            .clone()
            .or_else(|| event.station_name.clone());
        self.origin.market_id = event.market_id;
    }

    fn apply_travel(&mut self, event: &TravelEvent) {
        self.origin.system_name = event.star_system.clone();
        self.origin.system_address = event.system_address;
        self.origin.station_name = None;
        self.origin.market_id = None;
    }

    fn clear_station_origin(&mut self) {
        self.origin.station_name = None;
        self.origin.market_id = None;
    }

    fn apply_accepted(&mut self, event: &MissionEvent) {
        let Some(mission_id) = event.mission_id else {
            return;
        };

        self.missions.insert(
            mission_id,
            TrackedMission {
                mission_id,
                state: MissionState::Active,
                kind: mission_kind(event),
                name: event.name.clone(),
                localised_name: event.localised_name.clone(),
                issuing_faction: event.faction.clone(),
                target_faction: event.target_faction.clone(),
                accepted_at: event.timestamp,
                completion_time: None,
                expiry: event.expiry,
                reward: event.reward,
                influence: event.influence.clone(),
                reputation: event.reputation.clone(),
                wing: event.wing,
                origin: self.origin.clone(),
                destination_system: event.destination_system.clone(),
                destination_station: event.destination_station.clone(),
                destination_settlement: event.destination_settlement.clone(),
                progress: mission_progress(event),
            },
        );
    }

    fn update_mission(&mut self, event: &MissionEvent, state: MissionState) {
        let Some(mission_id) = event.mission_id else {
            return;
        };
        let Some(mission) = self.missions.get_mut(&mission_id) else {
            return;
        };

        mission.state = state;
        if matches!(
            state,
            MissionState::Completed | MissionState::Failed | MissionState::Abandoned
        ) {
            mission.completion_time = Some(event.timestamp);
        }
        if state == MissionState::Completed {
            mission.reward = completion_reward(event).or(mission.reward);
        }
        if matches!(state, MissionState::Failed | MissionState::Abandoned) {
            mission.reward = Some(0);
        }
        if state == MissionState::Redirected {
            mission.destination_system = event
                .new_destination_system
                .clone()
                .or_else(|| mission.destination_system.clone());
            mission.destination_station = event
                .new_destination_station
                .clone()
                .or_else(|| mission.destination_station.clone());
            if let MissionProgress::Massacre {
                kill_count, kills, ..
            } = &mut mission.progress
            {
                *kills = *kill_count;
            }
        }
    }

    pub(in crate::mission) fn retain_snapshot(&mut self, active_ids: impl Iterator<Item = u64>) {
        let active: BTreeSet<_> = active_ids.collect();

        self.missions.retain(|mission_id, mission| {
            !matches!(
                mission.state,
                MissionState::Active | MissionState::Redirected
            ) || active.contains(mission_id)
        });
    }

    fn apply_bounty(&mut self, event: &BountyEvent) {
        if event.total_reward.unwrap_or(0) == 0 {
            return;
        }
        if event
            .target
            .as_deref()
            .is_some_and(|target| target.to_ascii_lowercase().contains("suit"))
        {
            return;
        }
        let Some(victim_faction) = event.victim_faction.as_deref() else {
            return;
        };
        if invalid_massacre_victim(victim_faction) {
            return;
        }
        self.apply_massacre_kill(victim_faction);
    }

    fn apply_cargo_depot(&mut self, event: &CargoDepotEvent) {
        let Some(mission_id) = event.mission_id else {
            return;
        };
        let Some(mission) = self.missions.get_mut(&mission_id) else {
            return;
        };
        let MissionProgress::Trade {
            commodity,
            commodity_localised,
            count,
            items_collected,
            items_delivered,
        } = &mut mission.progress
        else {
            return;
        };

        if commodity.is_none() {
            *commodity = event.cargo_type.clone();
        }
        if commodity_localised.is_none() {
            *commodity_localised = event.cargo_type_localised.clone();
        }
        if *count == 0 {
            *count = event.total_items_to_deliver.or(event.count).unwrap_or(0);
        }
        if let Some(value) = event.items_collected {
            *items_collected = value;
        }
        if let Some(value) = event.items_delivered {
            *items_delivered = value;
        }
    }

    fn apply_massacre_kill(&mut self, victim_faction: &str) {
        let mut progressed_issuers = BTreeSet::new();
        for mission in self.missions.values_mut() {
            let MissionProgress::Massacre {
                target_faction,
                kill_count,
                kills,
                ..
            } = &mut mission.progress
            else {
                continue;
            };
            if mission.state != MissionState::Active || *kills >= *kill_count {
                continue;
            }
            if target_faction
                .as_deref()
                .is_some_and(|target| target.eq_ignore_ascii_case(victim_faction))
            {
                let issuer = mission.issuing_faction.clone().unwrap_or_default();
                if !progressed_issuers.insert(issuer) {
                    continue;
                }
                *kills += 1;
            }
        }
    }
}
