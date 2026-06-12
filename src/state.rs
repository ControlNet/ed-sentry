use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};

use crate::event::{
    BountyEvent, FactionKillBondEvent, HullDamageEvent, JournalEvent, LocationEvent, MissionEvent,
    MusicEvent, SupercruiseDestinationDropEvent,
};

pub const RECENT_RATE_WINDOW: Duration = Duration::minutes(10);

const MINIMUM_TOTAL_RATE_DURATION: Duration = Duration::seconds(1);

pub fn total_kill_rate_per_hour(
    kills: u64,
    session_started_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> f64 {
    total_event_rate_per_hour(kills, session_started_at, now)
}

pub fn total_scan_rate_per_hour(
    scans: u64,
    session_started_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> f64 {
    total_event_rate_per_hour(scans, session_started_at, now)
}

pub fn total_event_rate_per_hour(
    events: u64,
    session_started_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> f64 {
    let Some(started_at) = session_started_at else {
        return 0.0;
    };

    let elapsed = now.signed_duration_since(started_at);
    rate_per_hour(events, elapsed.max(MINIMUM_TOTAL_RATE_DURATION))
}

pub fn recent_kill_rate_per_hour(kill_timestamps: &[DateTime<Utc>], now: DateTime<Utc>) -> f64 {
    recent_event_rate_per_hour(kill_timestamps, now)
}

pub fn recent_scan_rate_per_hour(scan_timestamps: &[DateTime<Utc>], now: DateTime<Utc>) -> f64 {
    recent_event_rate_per_hour(scan_timestamps, now)
}

pub fn recent_event_rate_per_hour(event_timestamps: &[DateTime<Utc>], now: DateTime<Utc>) -> f64 {
    let window_start = now - RECENT_RATE_WINDOW;
    let events_in_window = event_timestamps
        .iter()
        .filter(|timestamp| **timestamp >= window_start && **timestamp <= now)
        .count() as u64;

    rate_per_hour(events_in_window, RECENT_RATE_WINDOW)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SessionState {
    pub commander: Option<String>,
    pub ship: Option<String>,
    pub system: Option<String>,
    pub mode: Option<String>,
    pub active_session: bool,
    pub session_started_at: Option<DateTime<Utc>>,
    pub session_ended_at: Option<DateTime<Utc>>,
    pub shields_up: Option<bool>,
    pub ship_hull: Option<f64>,
    pub fighter_hull: Option<f64>,
    pub fighter_alive: Option<bool>,
    pub cargo_scans: u64,
    pub kills: u64,
    pub bounty_total: u64,
    pub merits: u64,
    pub merits_to_report: u64,
    pub victim_faction_kills: BTreeMap<String, u64>,
    pub active_massacre_mission_ids: BTreeSet<u64>,
    pub mission_total: u64,
    pub mission_completed: u64,
    pub last_kill_at: Option<DateTime<Utc>>,
    pub last_scan_at: Option<DateTime<Utc>>,
    kill_timestamps: Vec<DateTime<Utc>>,
    scan_timestamps: Vec<DateTime<Utc>>,
}

impl SessionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_event(&mut self, event: &JournalEvent) {
        match event {
            JournalEvent::Commander(event) => assign_if_some(&mut self.commander, &event.name),
            JournalEvent::LoadGame(event) => {
                assign_if_some(&mut self.commander, &event.commander);
                assign_if_some(&mut self.ship, &event.ship);
                assign_if_some(&mut self.ship, &event.ship_localised);
                assign_if_some(&mut self.mode, &event.game_mode);
            }
            JournalEvent::Loadout(event) => {
                assign_if_some(&mut self.ship, &event.ship);
                assign_if_some(&mut self.ship, &event.ship_localised);
            }
            JournalEvent::Location(event) => self.apply_location(event),
            JournalEvent::SupercruiseDestinationDrop(event) => self.apply_destination_drop(event),
            JournalEvent::SupercruiseEntry(event) | JournalEvent::FSDJump(event) => {
                self.end_session(event.timestamp)
            }
            JournalEvent::Shutdown(event) | JournalEvent::Died(event) => {
                self.end_session(event.timestamp)
            }
            JournalEvent::Music(event) => self.apply_music(event),
            JournalEvent::ShipTargeted(_) => {}
            JournalEvent::Bounty(event) => self.apply_bounty(event),
            JournalEvent::FactionKillBond(event) => self.apply_faction_kill_bond(event),
            JournalEvent::MissionAccepted(event) => self.apply_mission_accepted(event),
            JournalEvent::MissionRedirected(event) => self.apply_mission_redirected(event),
            JournalEvent::MissionCompleted(event)
            | JournalEvent::MissionFailed(event)
            | JournalEvent::MissionAbandoned(event) => self.remove_active_massacre(event),
            JournalEvent::ShieldState(event) => self.shields_up = event.shields_up,
            JournalEvent::HullDamage(event) => self.apply_hull_damage(event),
            JournalEvent::FighterDestroyed(_) => {
                self.fighter_alive = Some(false);
                self.fighter_hull = Some(0.0);
            }
            JournalEvent::LaunchFighter(event) if event.player_controlled == Some(false) => {
                self.fighter_alive = Some(true);
                self.fighter_hull.get_or_insert(1.0);
            }
            JournalEvent::ReceiveText(_) => {}
            JournalEvent::Rank(_)
            | JournalEvent::Progress(_)
            | JournalEvent::Missions(_)
            | JournalEvent::LaunchFighter(_)
            | JournalEvent::StartJump(_)
            | JournalEvent::PowerplayMerits(_)
            | JournalEvent::EjectCargo(_)
            | JournalEvent::ReservoirReplenished(_)
            | JournalEvent::ShipyardSwap(_)
            | JournalEvent::StartupSnapshot(_)
            | JournalEvent::Station(_)
            | JournalEvent::Exploration(_)
            | JournalEvent::Navigation(_)
            | JournalEvent::CargoMaterial(_)
            | JournalEvent::ShipModule(_)
            | JournalEvent::MissionDetail(_)
            | JournalEvent::CombatDetail(_)
            | JournalEvent::Odyssey(_)
            | JournalEvent::Social(_)
            | JournalEvent::Powerplay(_)
            | JournalEvent::Squadron(_)
            | JournalEvent::Carrier(_)
            | JournalEvent::Colonisation(_)
            | JournalEvent::Unknown { .. } => {}
        }
    }

    pub fn total_kill_rate_per_hour_at(&self, now: DateTime<Utc>) -> f64 {
        total_kill_rate_per_hour(self.kills, self.active_start_for_rates(), now)
    }

    pub fn recent_kill_rate_per_hour_at(&self, now: DateTime<Utc>) -> f64 {
        recent_kill_rate_per_hour(&self.kill_timestamps, now)
    }

    pub fn total_scan_rate_per_hour_at(&self, now: DateTime<Utc>) -> f64 {
        total_scan_rate_per_hour(self.cargo_scans, self.active_start_for_rates(), now)
    }

    pub fn recent_scan_rate_per_hour_at(&self, now: DateTime<Utc>) -> f64 {
        recent_scan_rate_per_hour(&self.scan_timestamps, now)
    }

    pub fn kill_timestamps(&self) -> &[DateTime<Utc>] {
        &self.kill_timestamps
    }

    pub fn scan_timestamps(&self) -> &[DateTime<Utc>] {
        &self.scan_timestamps
    }

    pub fn reset_session_counters(&mut self) {
        self.active_session = false;
        self.session_started_at = None;
        self.session_ended_at = None;
        self.cargo_scans = 0;
        self.kills = 0;
        self.bounty_total = 0;
        self.merits = 0;
        self.merits_to_report = 0;
        self.victim_faction_kills.clear();
        self.last_kill_at = None;
        self.last_scan_at = None;
        self.kill_timestamps.clear();
        self.scan_timestamps.clear();
    }

    fn apply_location(&mut self, event: &LocationEvent) {
        assign_if_some(&mut self.system, &event.star_system);

        if location_is_planetary_ring(event) {
            self.start_session(event.timestamp);
        }
    }

    fn apply_destination_drop(&mut self, event: &SupercruiseDestinationDropEvent) {
        if destination_drop_starts_session(event) {
            self.reset_session_at(event.timestamp);
        }
    }

    fn apply_music(&mut self, event: &MusicEvent) {
        if text_equals(event.music_track.as_deref(), "mainmenu") {
            self.end_session(event.timestamp);
        }
    }

    fn apply_bounty(&mut self, event: &BountyEvent) {
        self.start_session_backdated(event.timestamp, Duration::seconds(60));
        self.record_kill(
            event.timestamp,
            bounty_reward_total(event),
            bounty_victim_faction(event),
        );
    }

    fn apply_faction_kill_bond(&mut self, event: &FactionKillBondEvent) {
        self.start_session_backdated(event.timestamp, Duration::seconds(60));
        self.record_kill(
            event.timestamp,
            event.reward.unwrap_or(0),
            faction_kill_bond_victim_faction(event),
        );
    }

    fn apply_mission_accepted(&mut self, event: &MissionEvent) {
        if !mission_is_massacre(event) {
            return;
        }

        self.mission_total += 1;
        if let Some(mission_id) = event.mission_id {
            self.active_massacre_mission_ids.insert(mission_id);
        }
    }

    fn apply_mission_redirected(&mut self, event: &MissionEvent) {
        let Some(mission_id) = event.mission_id else {
            return;
        };

        if self.active_massacre_mission_ids.contains(&mission_id) || mission_is_massacre(event) {
            self.mission_completed += 1;
        }
    }

    fn remove_active_massacre(&mut self, event: &MissionEvent) {
        if let Some(mission_id) = event.mission_id {
            self.active_massacre_mission_ids.remove(&mission_id);
        }
    }

    fn apply_hull_damage(&mut self, event: &HullDamageEvent) {
        if event.fighter == Some(true) {
            self.fighter_hull = event.health;
            if event.health == Some(0.0) {
                self.fighter_alive = Some(false);
            }
            return;
        }

        if event.player_pilot != Some(false) {
            self.ship_hull = event.health;
        }
    }

    pub fn start_session_at(&mut self, timestamp: DateTime<Utc>) {
        self.start_session(timestamp);
    }

    pub fn start_session_backdated(&mut self, timestamp: DateTime<Utc>, delta: Duration) {
        self.start_session(timestamp - delta);
    }

    pub fn reset_session_at(&mut self, timestamp: DateTime<Utc>) {
        self.active_session = false;
        self.clear_observed_session_stats();
        self.start_session(timestamp);
    }

    pub fn record_incoming_scan(&mut self, timestamp: DateTime<Utc>) {
        self.record_scan(timestamp);
    }

    pub fn record_merits(&mut self, merits: u64) {
        self.merits += merits;
        if self.merits_to_report > 0 {
            self.merits_to_report -= 1;
        }
    }

    fn start_session(&mut self, timestamp: DateTime<Utc>) {
        if self.active_session {
            return;
        }

        if self.session_ended_at.is_some() {
            self.clear_observed_session_stats();
        }
        self.active_session = true;
        self.session_started_at = Some(timestamp);
        self.session_ended_at = None;
    }

    fn end_session(&mut self, timestamp: DateTime<Utc>) {
        if !self.active_session {
            return;
        }

        self.active_session = false;
        self.session_ended_at = Some(timestamp);
    }

    fn record_kill(&mut self, timestamp: DateTime<Utc>, reward: u64, victim_faction: Option<&str>) {
        self.kills += 1;
        self.bounty_total += reward;
        self.merits_to_report += 1;
        self.last_kill_at = Some(timestamp);
        self.kill_timestamps.push(timestamp);
        if let Some(faction) = victim_faction.filter(|faction| !faction.is_empty()) {
            *self
                .victim_faction_kills
                .entry(faction.to_string())
                .or_default() += 1;
        }
    }

    fn record_scan(&mut self, timestamp: DateTime<Utc>) {
        self.cargo_scans += 1;
        self.last_scan_at = Some(timestamp);
        self.scan_timestamps.push(timestamp);
    }

    fn active_start_for_rates(&self) -> Option<DateTime<Utc>> {
        self.active_session
            .then_some(self.session_started_at)
            .flatten()
    }

    fn clear_observed_session_stats(&mut self) {
        self.cargo_scans = 0;
        self.kills = 0;
        self.bounty_total = 0;
        self.merits = 0;
        self.merits_to_report = 0;
        self.victim_faction_kills.clear();
        self.last_kill_at = None;
        self.last_scan_at = None;
        self.kill_timestamps.clear();
        self.scan_timestamps.clear();
    }
}

pub fn mission_is_massacre(event: &MissionEvent) -> bool {
    text_contains_any(event.name.as_deref(), &["mission_massacre"])
}

fn assign_if_some(destination: &mut Option<String>, source: &Option<String>) {
    if let Some(value) = source.as_ref().filter(|value| !value.is_empty()) {
        *destination = Some(value.clone());
    }
}

fn location_is_planetary_ring(event: &LocationEvent) -> bool {
    text_contains_any(event.body_type.as_deref(), &["planetaryring", "ring"])
        || text_contains_any(event.body.as_deref(), &[" ring"])
}

fn destination_drop_starts_session(event: &SupercruiseDestinationDropEvent) -> bool {
    text_contains_any(
        event.destination_type.as_deref(),
        &["$multiplayer", "$warzone", "resourceextraction"],
    )
}

fn bounty_reward_total(event: &BountyEvent) -> u64 {
    event
        .rewards
        .as_ref()
        .and_then(|rewards| rewards.first())
        .and_then(|reward| reward.reward)
        .or(event.total_reward)
        .unwrap_or(0)
}

fn bounty_victim_faction(event: &BountyEvent) -> Option<&str> {
    event
        .victim_faction_localised
        .as_deref()
        .or(event.victim_faction.as_deref())
}

fn faction_kill_bond_victim_faction(event: &FactionKillBondEvent) -> Option<&str> {
    event
        .victim_faction_localised
        .as_deref()
        .or(event.victim_faction.as_deref())
}

fn text_contains_any(text: Option<&str>, needles: &[&str]) -> bool {
    let Some(text) = text else {
        return false;
    };
    let lower_text = text.to_ascii_lowercase();
    needles.iter().any(|needle| lower_text.contains(needle))
}

fn text_equals(text: Option<&str>, expected: &str) -> bool {
    text.is_some_and(|text| text.eq_ignore_ascii_case(expected))
}

fn rate_per_hour(events: u64, duration: Duration) -> f64 {
    if events == 0 {
        return 0.0;
    }

    let duration_hours = duration.num_milliseconds() as f64 / 3_600_000.0;
    events as f64 / duration_hours
}

#[cfg(test)]
mod time_stats_rates {
    use super::*;
    use chrono::TimeZone;

    fn utc_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 9, 12, 0, 0).single().unwrap()
    }

    #[test]
    fn inactive_session_total_rates_are_zero() {
        let now = utc_timestamp();

        assert_eq!(total_kill_rate_per_hour(10, None, now), 0.0);
        assert_eq!(total_scan_rate_per_hour(20, None, now), 0.0);
    }

    #[test]
    fn zero_duration_total_rate_is_finite() {
        let now = utc_timestamp();

        let rate = total_kill_rate_per_hour(2, Some(now), now);

        assert!(rate.is_finite());
        assert_eq!(rate, 7200.0);
    }

    #[test]
    fn one_hour_total_rates_match_counts() {
        let now = utc_timestamp();
        let started_at = now - Duration::hours(1);

        assert_eq!(total_kill_rate_per_hour(42, Some(started_at), now), 42.0);
        assert_eq!(total_scan_rate_per_hour(120, Some(started_at), now), 120.0);
    }

    #[test]
    fn ten_minute_recent_rate_uses_rolling_window() {
        let now = utc_timestamp();
        let timestamps = vec![
            now - Duration::minutes(11),
            now - Duration::minutes(10),
            now - Duration::minutes(9),
            now,
            now + Duration::seconds(1),
        ];

        assert_eq!(recent_kill_rate_per_hour(&timestamps, now), 18.0);
        assert_eq!(recent_scan_rate_per_hour(&timestamps, now), 18.0);
    }
}
