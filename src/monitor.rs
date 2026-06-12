use std::collections::VecDeque;

use chrono::{DateTime, Duration, Utc};

use crate::config::{LogLevelConfig, MonitorConfig, RuntimeConfig};
use crate::event::{
    BountyEvent, FactionKillBondEvent, JournalEvent, MissionEvent, ReservoirReplenishedEvent,
    ShipTargetedEvent,
};
use crate::notifier::{AlertLevel, Notification, NotificationDispatcher, Notifier};
use crate::state::SessionState;
use crate::terminal::{render_dynamic_title, render_monitor_status_line};
use crate::text::line_safe;

#[derive(Clone, Debug)]
pub struct EventMonitor<N> {
    state: SessionState,
    dispatcher: NotificationDispatcher<N>,
    monitor_config: MonitorConfig,
    log_levels: LogLevelConfig,
    warnings: WarningScheduler,
    recent_incoming_scans: VecDeque<String>,
    recent_outgoing_scans: VecDeque<String>,
    last_security_ship: Option<String>,
    combat_rank: Option<String>,
    combat_progress: Option<u8>,
    fuel_capacity: f64,
    last_fuel_at: Option<DateTime<Utc>>,
    last_fuel_main: Option<f64>,
    mission_tracking_loaded: bool,
    active_massacre_missions: Vec<u64>,
    mission_redirects: u64,
    last_event_name: Option<String>,
}

const FUEL_LOW: f64 = 0.2;
const FUEL_CRITICAL: f64 = 0.1;
const DEFAULT_FUEL_CAPACITY: f64 = 64.0;
const COMBAT_RANKS: &[&str] = &[
    "Harmless",
    "Mostly Harmless",
    "Novice",
    "Competent",
    "Expert",
    "Master",
    "Dangerous",
    "Deadly",
    "Elite",
    "Elite I",
    "Elite II",
    "Elite III",
    "Elite IV",
    "Elite V",
];
const SHIPS_EASY: &[&str] = &[
    "adder",
    "asp",
    "asp_scout",
    "cobramkiii",
    "cobramkiv",
    "diamondback",
    "diamondbackxl",
    "eagle",
    "empire_courier",
    "empire_eagle",
    "krait_light",
    "sidewinder",
    "viper",
    "viper_mkiv",
];
const SHIPS_HARD: &[&str] = &[
    "typex",
    "typex_2",
    "typex_3",
    "anaconda",
    "federation_dropship_mkii",
    "federation_dropship",
    "federation_gunship",
    "ferdelance",
    "empire_trader",
    "krait_mkii",
    "python",
    "vulture",
    "type9_military",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct WarningScheduler {
    initial_no_kills_sent: bool,
    last_no_kills_warning_at: Option<DateTime<Utc>>,
    last_low_rate_warning_at: Option<DateTime<Utc>>,
    cooldown_multiplier: i32,
}

impl<N: Notifier> EventMonitor<N> {
    pub fn new(notifier: N, monitor_config: MonitorConfig, log_levels: LogLevelConfig) -> Self {
        let dispatcher =
            NotificationDispatcher::from_config(notifier, &monitor_config, &log_levels);
        Self {
            state: SessionState::new(),
            dispatcher,
            monitor_config,
            log_levels,
            warnings: WarningScheduler::default(),
            recent_incoming_scans: VecDeque::new(),
            recent_outgoing_scans: VecDeque::new(),
            last_security_ship: None,
            combat_rank: None,
            combat_progress: None,
            fuel_capacity: DEFAULT_FUEL_CAPACITY,
            last_fuel_at: None,
            last_fuel_main: None,
            mission_tracking_loaded: false,
            active_massacre_missions: Vec::new(),
            mission_redirects: 0,
            last_event_name: None,
        }
    }

    pub fn from_runtime_config(notifier: N, config: &RuntimeConfig) -> Self {
        Self::new(notifier, config.monitor.clone(), config.log_levels.clone())
    }

    pub fn process_event(&mut self, event: &JournalEvent) -> anyhow::Result<()> {
        let previous_state = self.state.clone();
        self.state.apply_event(event);

        if self.state.session_started_at != previous_state.session_started_at {
            self.warnings.reset_for_session();
        }
        if self.state.kills > previous_state.kills {
            self.warnings.record_kill();
        }

        for notification in self.notifications_for_event(event, &previous_state) {
            self.dispatcher.dispatch(notification)?;
        }
        self.last_event_name = Some(event.event_name().to_string());

        Ok(())
    }

    pub fn check_warnings_at(&mut self, now: DateTime<Utc>, preload: bool) -> anyhow::Result<()> {
        if preload || !self.state.active_session {
            return Ok(());
        }

        let Some(session_started_at) = self.state.session_started_at else {
            return Ok(());
        };

        self.warnings.clear_elapsed(now, self.warning_cooldown());

        let warning = if self.state.kills == 0 {
            self.no_kills_warning(now, session_started_at)
        } else if self.low_kill_rate_is_below_threshold(now) {
            self.low_kill_rate_warning(now)
        } else {
            self.no_kills_warning(now, session_started_at)
        };
        if let Some(notification) = warning {
            self.dispatcher.dispatch(notification)?;
        }

        Ok(())
    }

    pub fn state(&self) -> &SessionState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut SessionState {
        &mut self.state
    }

    pub fn dispatcher(&self) -> &NotificationDispatcher<N> {
        &self.dispatcher
    }

    pub fn dispatcher_mut(&mut self) -> &mut NotificationDispatcher<N> {
        &mut self.dispatcher
    }

    pub fn into_dispatcher(self) -> NotificationDispatcher<N> {
        self.dispatcher
    }

    pub fn finish(&mut self, journal_file: &str, timestamp: DateTime<Utc>) -> anyhow::Result<()> {
        if let Some(summary) = self.summary_notification(false, timestamp) {
            self.dispatcher.dispatch(summary)?;
        }
        self.dispatcher.dispatch(self.notification(
            "monitor_stopped",
            2,
            format!("Monitor stopped ({journal_file})"),
            timestamp,
        ))?;
        Ok(())
    }

    pub fn start_monitor(
        &mut self,
        journal_file: &str,
        timestamp: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        self.dispatcher.dispatch(self.notification(
            "monitor_started",
            2,
            format!("Monitor started ({journal_file})"),
            timestamp,
        ))
    }

    pub fn render_status_line(&self, now: DateTime<Utc>) -> String {
        render_monitor_status_line(
            &self.state,
            &self.monitor_config,
            now,
            self.mission_redirects,
            self.active_massacre_missions.len(),
        )
    }

    pub fn render_dynamic_title(&self, now: DateTime<Utc>) -> String {
        render_dynamic_title(
            &self.state,
            &self.monitor_config,
            now,
            self.mission_redirects,
            self.active_massacre_missions.len(),
        )
    }

    fn notifications_for_event(
        &mut self,
        event: &JournalEvent,
        previous_state: &SessionState,
    ) -> Vec<Notification> {
        let mut notifications = Vec::new();
        match event {
            JournalEvent::Rank(rank) => {
                self.combat_rank = rank.combat.and_then(combat_rank_name).map(str::to_string);
            }
            JournalEvent::Progress(progress) => {
                self.combat_progress = progress.combat;
            }
            JournalEvent::Loadout(loadout) => {
                if let Some(capacity) = loadout
                    .fuel_capacity_main
                    .filter(|capacity| *capacity >= 2.0)
                {
                    self.fuel_capacity = capacity;
                }
            }
            JournalEvent::LoadGame(load_game) => notifications.push(self.notification(
                "commander_load",
                2,
                self.commander_text(load_game),
                load_game.timestamp,
            )),
            JournalEvent::SupercruiseDestinationDrop(drop) => {
                if text_contains_any(
                    drop.destination_type.as_deref(),
                    &["$multiplayer", "$warzone"],
                ) || text_contains_any(
                    drop.destination_type_localised.as_deref(),
                    &["resource extraction", "combat"],
                ) {
                    let place = clean_text(
                        drop.destination_type_localised
                            .as_deref()
                            .or(drop.destination_type.as_deref())
                            .unwrap_or("[Unknown]"),
                    );
                    notifications.push(self.notification(
                        "destination_drop",
                        2,
                        format!("Dropped at {place}"),
                        drop.timestamp,
                    ));
                }
            }
            JournalEvent::SupercruiseEntry(entry) => notifications.push(self.notification(
                "supercruise_entry",
                2,
                format!(
                    "Supercruise entry in {}",
                    clean_text(entry.star_system.as_deref().unwrap_or("[Unknown]"))
                ),
                entry.timestamp,
            )),
            JournalEvent::FSDJump(jump) => notifications.push(self.notification(
                "fsd_jump",
                2,
                format!(
                    "FSD jump to {}",
                    clean_text(jump.star_system.as_deref().unwrap_or("[Unknown]"))
                ),
                jump.timestamp,
            )),
            JournalEvent::Music(music) if text_equals(music.music_track.as_deref(), "MainMenu") => {
                notifications.push(self.notification(
                    "main_menu",
                    2,
                    "Exited to main menu".to_string(),
                    music.timestamp,
                ));
            }
            JournalEvent::ShipyardSwap(swap) => notifications.push(self.notification(
                "shipyard_swap",
                2,
                format!(
                    "Swapped ship to {}",
                    ship_name(
                        swap.ship_type.as_deref(),
                        swap.ship_type_localised.as_deref()
                    )
                ),
                swap.timestamp,
            )),
            JournalEvent::ShipTargeted(targeted) => {
                if let Some(notification) = self.ship_targeted_notification(targeted) {
                    notifications.push(notification);
                }
            }
            JournalEvent::ReceiveText(text) => {
                let message = text.message.as_deref().unwrap_or("");
                if text.channel.as_deref() == Some("npc")
                    && message.contains("$Pirate_OnStartScanCargo")
                {
                    let sender = clean_text(
                        text.from_localised
                            .as_deref()
                            .or(text.from.as_deref())
                            .unwrap_or("[Unknown]"),
                    );
                    if self.recent_incoming_scans.contains(&sender) {
                        return notifications;
                    }
                    push_recent(&mut self.recent_incoming_scans, sender.clone(), 5);
                    self.state.record_incoming_scan(text.timestamp);
                    let mut output = "Cargo scan".to_string();
                    if self.monitor_config.pirate_names {
                        output.push_str(&format!(" [{sender}]"));
                    }
                    notifications.push(self.notification(
                        "cargo_scan",
                        self.log_levels.scan_incoming,
                        output,
                        text.timestamp,
                    ));
                } else if message.contains("Police_Attack") {
                    notifications.push(self.notification(
                        "security_attack",
                        self.log_levels.security_attack,
                        "Under attack by security services!".to_string(),
                        text.timestamp,
                    ));
                } else if [
                    "$Pirate_ThreatTooHigh",
                    "$Pirate_NotEnoughCargo",
                    "$Pirate_OnNoCargoFound",
                ]
                .iter()
                .any(|needle| message.contains(needle))
                {
                    notifications.push(self.notification(
                        "bait_value_low",
                        self.log_levels.bait_value_low,
                        "Pirate didn't engage due to insufficient cargo value".to_string(),
                        text.timestamp,
                    ));
                }
            }
            JournalEvent::Bounty(bounty) => {
                notifications.push(self.kill_notification(
                    "kill_bounty",
                    self.kill_level(bounty.target.as_deref()),
                    self.bounty_text(bounty, previous_state),
                    bounty.timestamp,
                ));
                if self.state.kills > 0 && self.state.kills.is_multiple_of(10) {
                    if let Some(summary) = self.summary_notification(true, bounty.timestamp) {
                        notifications.push(summary);
                    }
                }
            }
            JournalEvent::FactionKillBond(kill_bond) => {
                notifications.push(self.kill_notification(
                    "kill_bond",
                    self.log_levels.kill_hard,
                    self.kill_bond_text(kill_bond, previous_state),
                    kill_bond.timestamp,
                ));
                if self.state.kills > 0 && self.state.kills.is_multiple_of(10) {
                    if let Some(summary) = self.summary_notification(true, kill_bond.timestamp) {
                        notifications.push(summary);
                    }
                }
            }
            JournalEvent::MissionRedirected(mission) => {
                if self.state.mission_completed > previous_state.mission_completed {
                    notifications.push(self.mission_redirect_notification(mission));
                }
            }
            JournalEvent::MissionAccepted(mission) => {
                if self.mission_tracking_loaded
                    && mission_is_massacre_text(
                        mission.name.as_deref(),
                        mission.localised_name.as_deref(),
                    )
                {
                    if let Some(mission_id) = mission.mission_id {
                        self.active_massacre_missions.push(mission_id);
                    }
                    notifications.push(self.notification(
                        "mission_accepted",
                        self.log_levels.missions,
                        format!(
                            "Accepted massacre mission (active: {})",
                            self.active_massacre_missions.len()
                        ),
                        mission.timestamp,
                    ));
                }
            }
            JournalEvent::MissionCompleted(mission)
            | JournalEvent::MissionFailed(mission)
            | JournalEvent::MissionAbandoned(mission) => {
                if self
                    .active_massacre_missions
                    .contains(&mission.mission_id.unwrap_or(0))
                {
                    if let Some(mission_id) = mission.mission_id {
                        self.active_massacre_missions
                            .retain(|active| *active != mission_id);
                    }
                    if self.mission_redirects > 0 {
                        self.mission_redirects -= 1;
                    }
                    let event = mission
                        .event
                        .strip_prefix("Mission")
                        .unwrap_or(&mission.event)
                        .to_ascii_lowercase();
                    notifications.push(self.notification(
                        "mission_status",
                        self.log_levels.missions,
                        format!(
                            "Massacre mission {event} (active: {})",
                            self.active_massacre_missions.len()
                        ),
                        mission.timestamp,
                    ));
                }
            }
            JournalEvent::Missions(event) => {
                if self.mission_tracking_loaded {
                    return notifications;
                }
                self.mission_tracking_loaded = true;
                self.mission_redirects = 0;
                self.active_massacre_missions = event
                    .active
                    .iter()
                    .filter(|mission| {
                        mission.expires.unwrap_or(0) > 0
                            && mission_is_massacre_text(mission.name.as_deref(), None)
                    })
                    .filter_map(|mission| mission.mission_id)
                    .collect();
                notifications.push(self.notification(
                    "missions_snapshot",
                    self.log_levels.missions,
                    format!(
                        "Missions loaded (active massacres: {})",
                        self.active_massacre_missions.len()
                    ),
                    event.timestamp,
                ));
            }
            JournalEvent::ShieldState(shields) => {
                if let Some(shields_up) = shields.shields_up {
                    let text = if shields_up {
                        "Ship shields back up"
                    } else {
                        "Ship shields down!"
                    };
                    notifications.push(self.notification(
                        "ship_shields",
                        self.log_levels.ship_shields,
                        text.to_string(),
                        shields.timestamp,
                    ));
                }
            }
            JournalEvent::HullDamage(hull) => {
                if let Some(health) = hull.health {
                    if hull.fighter == Some(true) && hull.player_pilot == Some(false) {
                        if previous_state.fighter_hull == Some(health) {
                            return notifications;
                        }
                        notifications.push(self.notification(
                            "fighter_hull",
                            self.log_levels.fighter_hull,
                            format!("Fighter hull damaged! (Integrity: {}%)", percentage(health)),
                            hull.timestamp,
                        ));
                    } else if hull.player_pilot == Some(true) && hull.fighter == Some(false) {
                        notifications.push(self.notification(
                            "ship_hull",
                            self.log_levels.ship_hull,
                            format!("Ship hull damaged! (Integrity: {}%)", percentage(health)),
                            hull.timestamp,
                        ));
                    }
                }
            }
            JournalEvent::FighterDestroyed(event)
                if self.last_event_name.as_deref() != Some("StartJump") =>
            {
                notifications.push(self.notification(
                    "fighter_destroyed",
                    self.log_levels.fighter_down,
                    "Fighter destroyed!".to_string(),
                    event.timestamp,
                ))
            }
            JournalEvent::LaunchFighter(event) if event.player_controlled == Some(false) => {
                notifications.push(self.notification(
                    "fighter_launch",
                    2,
                    "Fighter launched".to_string(),
                    event.timestamp,
                ))
            }
            JournalEvent::PowerplayMerits(event) => {
                if self.state.merits_to_report > 0 && event.merits_gained.unwrap_or(0) < 500 {
                    let merits = event.merits_gained.unwrap_or(0);
                    self.state.record_merits(merits);
                    notifications.push(self.notification(
                        "merits",
                        self.log_levels.merits,
                        format!(
                            "Merits: +{} ({})",
                            merits,
                            clean_text(event.power.as_deref().unwrap_or("[Unknown]"))
                        ),
                        event.timestamp,
                    ));
                }
            }
            JournalEvent::EjectCargo(cargo)
                if cargo.abandoned == Some(false) && cargo.count == Some(1) =>
            {
                notifications.push(self.notification(
                    "cargo_lost",
                    self.log_levels.cargo_lost,
                    format!(
                            "Cargo stolen! ({})",
                            clean_text(
                                cargo
                                    .cargo_type_localised
                                    .as_deref()
                                    .or(cargo.cargo_type.as_deref())
                                    .unwrap_or("cargo")
                            )
                        ),
                    cargo.timestamp,
                ))
            }
            JournalEvent::ReservoirReplenished(fuel) => {
                let level = self.fuel_level(fuel.fuel_main);
                let text = self.fuel_text(fuel);
                notifications.push(self.notification("fuel_report", level, text, fuel.timestamp));
            }
            JournalEvent::Died(event) => notifications.push(self.notification(
                "died",
                self.log_levels.died,
                "Ship destroyed!".to_string(),
                event.timestamp,
            )),
            JournalEvent::Shutdown(event) => notifications.push(self.notification(
                "shutdown",
                2,
                "Quit to desktop".to_string(),
                event.timestamp,
            )),
            _ => {}
        }
        notifications
    }

    fn no_kills_warning(
        &mut self,
        now: DateTime<Utc>,
        session_started_at: DateTime<Utc>,
    ) -> Option<Notification> {
        if self.state.kills == 0 {
            if self.warnings.initial_no_kills_sent
                || now.signed_duration_since(session_started_at) < self.initial_no_kills_threshold()
            {
                return None;
            }

            self.warnings.initial_no_kills_sent = true;
            self.warnings.last_no_kills_warning_at = Some(now);
            let elapsed = now.signed_duration_since(session_started_at);
            return Some(self.notification(
                "no_kills",
                self.log_levels.no_kills,
                format!("No kills logged for {} minutes", elapsed.num_minutes()),
                now,
            ));
        }

        let last_kill_at = self.state.last_kill_at?;
        let elapsed_since_kill = now.signed_duration_since(last_kill_at);
        if elapsed_since_kill < self.later_no_kills_threshold()
            || self.warnings.last_no_kills_warning_at.is_some()
        {
            return None;
        }

        self.warnings.last_no_kills_warning_at = Some(now);
        Some(self.notification(
            "no_kills",
            self.log_levels.no_kills,
            format!(
                "Last logged kill was {} minutes ago",
                elapsed_since_kill.num_minutes()
            ),
            now,
        ))
    }

    fn low_kill_rate_warning(&mut self, now: DateTime<Utc>) -> Option<Notification> {
        if self.warnings.last_low_rate_warning_at.is_some() {
            return None;
        }

        if !self.low_kill_rate_is_below_threshold(now) {
            return None;
        }

        let rate = self.state.total_kill_rate_per_hour_at(now);
        self.warnings.last_low_rate_warning_at = Some(now);
        Some(self.notification(
            "kill_rate",
            self.log_levels.kill_rate,
            format!(
                "Kill rate of {:.1}/h is below {}/h threshold",
                rate, self.monitor_config.warn_kill_rate
            ),
            now,
        ))
    }

    fn low_kill_rate_is_below_threshold(&self, now: DateTime<Utc>) -> bool {
        if self.state.kills == 0 {
            return false;
        }

        let Some(session_started_at) = self.state.session_started_at else {
            return false;
        };
        if now.signed_duration_since(session_started_at) < self.kill_rate_delay_threshold() {
            return false;
        }

        self.state.total_kill_rate_per_hour_at(now) < f64::from(self.monitor_config.warn_kill_rate)
    }

    fn initial_no_kills_threshold(&self) -> Duration {
        Duration::minutes(i64::from(self.monitor_config.warn_no_kills_initial_minutes))
    }

    fn later_no_kills_threshold(&self) -> Duration {
        Duration::minutes(i64::from(self.monitor_config.warn_no_kills_minutes))
    }

    fn kill_rate_delay_threshold(&self) -> Duration {
        Duration::minutes(i64::from(self.monitor_config.warn_kill_rate_delay_minutes))
    }

    fn warning_cooldown(&self) -> Duration {
        Duration::minutes(i64::from(self.monitor_config.warn_cooldown_minutes))
            * self.warnings.cooldown_multiplier.max(1)
    }

    fn fuel_level(&self, fuel_main: Option<f64>) -> u8 {
        match fuel_main.map(|main| main / self.fuel_capacity) {
            Some(value) if value < FUEL_CRITICAL => self.log_levels.fuel_critical,
            Some(value) if value < FUEL_LOW => self.log_levels.fuel_low,
            Some(_) if self.state.active_session => self.log_levels.fuel_report,
            _ => 0,
        }
    }

    fn fuel_text(&mut self, event: &ReservoirReplenishedEvent) -> String {
        let Some(main) = event.fuel_main else {
            return "Fuel: unknown remaining".to_string();
        };

        let percentage = (main / self.fuel_capacity * 100.0).clamp(0.0, 100.0);
        let eta = if self.state.active_session {
            self.fuel_eta(main, event.timestamp)
        } else {
            None
        };

        self.last_fuel_at = Some(event.timestamp);
        self.last_fuel_main = Some(main);

        match eta {
            Some(eta) => format!("Fuel: {percentage:.0}% remaining (~{})", afk_duration(eta)),
            None => format!("Fuel: {percentage:.0}% remaining"),
        }
    }

    fn fuel_eta(&self, fuel_main: f64, timestamp: DateTime<Utc>) -> Option<Duration> {
        let previous_time = self.last_fuel_at?;
        let previous_fuel = self.last_fuel_main?;
        let elapsed_seconds = timestamp.signed_duration_since(previous_time).num_seconds();
        let consumed = previous_fuel - fuel_main;
        if elapsed_seconds <= 0 || consumed <= 0.0 || fuel_main <= 0.0 {
            return None;
        }

        Some(Duration::seconds(
            (fuel_main * elapsed_seconds as f64 / consumed) as i64,
        ))
    }

    fn commander_text(&self, event: &crate::event::LoadGameEvent) -> String {
        let commander = event
            .commander
            .as_deref()
            .or(self.state.commander.as_deref())
            .map(clean_text)
            .unwrap_or_else(|| "[Unknown]".to_string());
        let ship = ship_name(event.ship.as_deref(), event.ship_localised.as_deref());
        let mode = event
            .game_mode
            .as_deref()
            .map(display_game_mode)
            .unwrap_or("[Unknown]".to_string());
        let rank = self.combat_rank.as_deref().unwrap_or("[Unknown]");
        let progress = self
            .combat_progress
            .map(|progress| progress.to_string())
            .unwrap_or_else(|| "None".to_string());
        format!("CMDR {commander} ({ship} / {mode} / {rank} +{progress}%)")
    }

    fn ship_targeted_notification(&mut self, event: &ShipTargetedEvent) -> Option<Notification> {
        let ship_id = event.ship.as_deref()?;
        let ship = ship_name(Some(ship_id), event.ship_localised.as_deref());
        let pilot_name = event.pilot_name.as_deref().unwrap_or("");
        if pilot_name.contains("$ShipName_Police") {
            if self.last_security_ship.as_deref() == Some(ship.as_str()) {
                return None;
            }
            self.last_security_ship = Some(ship.clone());
            return Some(self.notification(
                "security_scan",
                self.log_levels.security_scan,
                format!("Scanned security ({ship})"),
                event.timestamp,
            ));
        }

        if !is_known_target_ship(ship_id) {
            return None;
        }
        self.state
            .start_session_backdated(event.timestamp, Duration::seconds(30));
        let scan_stage = event.scan_stage.unwrap_or(0);
        let check = if self.monitor_config.min_scan_level == 0 {
            ship.clone()
        } else {
            event
                .pilot_name_localised
                .as_deref()
                .map(clean_text)
                .unwrap_or_else(|| "[Unknown]".to_string())
        };
        if scan_stage < self.monitor_config.min_scan_level
            || self.recent_outgoing_scans.contains(&check)
        {
            return None;
        }
        push_recent(&mut self.recent_outgoing_scans, check.clone(), 10);

        let rank = event
            .pilot_rank
            .as_deref()
            .map(|rank| format!(" ({})", clean_text(rank)))
            .unwrap_or_default();
        let pirate = if self.monitor_config.pirate_names && check != "[Unknown]" {
            format!(" [{check}]")
        } else {
            String::new()
        };
        let level = if is_hard_ship(ship_id) {
            self.log_levels.scan_hard
        } else {
            self.log_levels.scan_easy
        };
        Some(self.notification(
            "ship_scan",
            level,
            format!("Scan: {ship}{rank}{pirate}"),
            event.timestamp,
        ))
    }

    fn kill_level(&self, ship_id: Option<&str>) -> u8 {
        if ship_id.is_some_and(is_hard_ship) {
            self.log_levels.kill_hard
        } else {
            self.log_levels.kill_easy
        }
    }

    fn mission_redirect_notification(&mut self, mission: &MissionEvent) -> Notification {
        self.mission_redirects += 1;
        let (text, level) = if self.active_massacre_missions.len() as u64 == self.mission_redirects
        {
            (
                format!(
                    "Completed kills for all missions! ({}/{})",
                    self.mission_redirects,
                    self.active_massacre_missions.len()
                ),
                self.log_levels.missions_all,
            )
        } else {
            (
                format!(
                    "Completed kills for a mission ({}/{})",
                    self.mission_redirects,
                    self.active_massacre_missions.len()
                ),
                self.log_levels.missions,
            )
        };
        self.notification("mission_redirected", level, text, mission.timestamp)
    }

    fn kill_notification(
        &self,
        event_type: &str,
        level: u8,
        text: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Notification {
        self.notification(event_type, level, text, timestamp)
    }

    fn bounty_text(&self, event: &BountyEvent, previous_state: &SessionState) -> String {
        let target = ship_name(event.target.as_deref(), event.target_localised.as_deref());
        let mut text = format!(
            "Kill{}: {target}",
            extended_count(self.monitor_config.extended_stats, self.state.kills)
        );
        if let Some(last_kill_at) = previous_state.last_kill_at {
            text.push_str(&format!(
                " (+{})",
                afk_duration(event.timestamp.signed_duration_since(last_kill_at))
            ));
        }
        if self.monitor_config.pirate_names {
            if let Some(pilot) = event.pilot_name_localised.as_deref() {
                text.push_str(&format!(" [{}]", truncate_text(&clean_text(pilot), 25)));
            }
        }
        if self.monitor_config.bounty_value {
            text.push_str(&format!(" [{} cr]", compact_number(bounty_credits(event))));
        }
        if self.monitor_config.bounty_faction {
            if let Some(faction) = event
                .victim_faction_localised
                .as_deref()
                .or(event.victim_faction.as_deref())
            {
                text.push_str(&format!(" [{}]", truncate_text(&clean_text(faction), 30)));
            }
        }
        text
    }

    fn kill_bond_text(
        &self,
        event: &FactionKillBondEvent,
        previous_state: &SessionState,
    ) -> String {
        let mut text = format!(
            "Kill{}: Bond",
            extended_count(self.monitor_config.extended_stats, self.state.kills)
        );
        if let Some(last_kill_at) = previous_state.last_kill_at {
            text.push_str(&format!(
                " (+{})",
                afk_duration(event.timestamp.signed_duration_since(last_kill_at))
            ));
        }
        if self.monitor_config.bounty_value {
            text.push_str(&format!(
                " [{} cr]",
                compact_number(event.reward.unwrap_or(0))
            ));
        }
        if self.monitor_config.bounty_faction {
            if let Some(faction) = event
                .victim_faction_localised
                .as_deref()
                .or(event.victim_faction.as_deref())
            {
                text.push_str(&format!(" [{}]", truncate_text(&clean_text(faction), 30)));
            }
        }
        text
    }

    fn summary_notification(
        &self,
        session: bool,
        timestamp: DateTime<Utc>,
    ) -> Option<Notification> {
        if self.state.kills < 2 {
            return None;
        }
        let log_max = self
            .log_levels
            .summary_kills
            .max(self.log_levels.summary_faction)
            .max(self.log_levels.summary_scans)
            .max(self.log_levels.summary_bounties)
            .max(self.log_levels.summary_merits);
        if log_max == 0 {
            return None;
        }
        let kill_times = self.state.kill_timestamps();
        let duration = kill_times
            .last()?
            .signed_duration_since(*kill_times.first()?);
        let average_seconds = duration.num_seconds().max(1) as f64 / (self.state.kills - 1) as f64;
        let hourly_rate = 3600.0 / average_seconds;
        let mut lines = Vec::new();
        if self.log_levels.summary_kills > 0 {
            lines.push(format!(
                "          -> Kills: {} ({:.1}/h | {})",
                self.state.kills,
                hourly_rate,
                afk_duration(Duration::seconds(average_seconds as i64))
            ));
        }
        if self.log_levels.summary_bounties > 0 {
            let bounties_hour = if duration.num_seconds() > 0 {
                self.state.bounty_total as f64 / (duration.num_seconds() as f64 / 3600.0)
            } else {
                0.0
            };
            let average = if self.state.kills > 0 {
                self.state.bounty_total / self.state.kills
            } else {
                0
            };
            lines.push(format!(
                "          -> Bounties: {} ({}/h | {}/kill)",
                compact_number(self.state.bounty_total),
                compact_number(bounties_hour.round() as u64),
                compact_number(average)
            ));
        }
        if self.log_levels.summary_scans > 0 && self.state.cargo_scans > 1 {
            let scan_times = self.state.scan_timestamps();
            if let (Some(first), Some(last)) = (scan_times.first(), scan_times.last()) {
                let scan_duration = last.signed_duration_since(*first);
                let average_seconds =
                    scan_duration.num_seconds().max(1) as f64 / (self.state.cargo_scans - 1) as f64;
                lines.push(format!(
                    "          -> Scans: {} ({:.1}/h | {})",
                    self.state.cargo_scans,
                    3600.0 / average_seconds,
                    afk_duration(Duration::seconds(average_seconds as i64))
                ));
            }
        }
        if self.log_levels.summary_faction > 0 {
            if let Some((faction, kills)) = self
                .state
                .victim_faction_kills
                .iter()
                .max_by_key(|(_, kills)| *kills)
                .filter(|(_, kills)| **kills > 1)
            {
                let faction_rate = if duration.num_seconds() > 0 && *kills > 1 {
                    3600.0 / (duration.num_seconds() as f64 / (*kills - 1) as f64)
                } else {
                    0.0
                };
                let percent = ((*kills as f64 / self.state.kills as f64) * 100.0).round() as u64;
                lines.push(format!(
                    "          -> Faction: {} ({:.1}/h | {}%) [{}]",
                    kills,
                    faction_rate,
                    percent,
                    truncate_text(&clean_text(faction), 30)
                ));
            }
        }
        if self.log_levels.summary_merits > 0 && self.state.merits > 0 {
            let merits_hour = if duration.num_seconds() > 0 {
                self.state.merits as f64 / (duration.num_seconds() as f64 / 3600.0)
            } else {
                0.0
            };
            let merits_average = self.state.merits as f64 / self.state.kills as f64;
            lines.push(format!(
                "          -> Merits: {} ({}/h | {}/kill)",
                self.state.merits,
                compact_number(merits_hour.round() as u64),
                compact_decimal(merits_average)
            ));
        }
        if lines.is_empty() {
            return None;
        }
        let title = if session { "Session" } else { "Total" };
        let missions = if self.active_massacre_missions.is_empty() {
            String::new()
        } else {
            format!(
                " [{}/{}]",
                self.mission_redirects,
                self.active_massacre_missions.len()
            )
        };
        let mut text = format!("{title} Stats ({}){missions}", afk_duration(duration));
        for line in lines {
            text.push('\n');
            text.push_str(&line);
        }
        Some(self.notification("summary", log_max, text, timestamp))
    }

    fn notification(
        &self,
        event_type: &str,
        level: u8,
        text: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Notification {
        let text = if event_type == "summary" {
            text
        } else {
            clean_text(&text)
        };
        Notification::new(
            event_type,
            level,
            alert_level(level),
            emoji_for_event(event_type).map(str::to_string),
            text.clone(),
            text,
            timestamp,
        )
    }
}

impl WarningScheduler {
    fn reset_for_session(&mut self) {
        *self = Self::default();
    }

    fn record_kill(&mut self) {
        self.initial_no_kills_sent = true;
    }

    fn clear_elapsed(&mut self, now: DateTime<Utc>, cooldown: Duration) {
        let mut cleared = false;
        if self
            .last_no_kills_warning_at
            .is_some_and(|last_warning_at| now.signed_duration_since(last_warning_at) >= cooldown)
        {
            self.last_no_kills_warning_at = None;
            cleared = true;
        }
        if self
            .last_low_rate_warning_at
            .is_some_and(|last_warning_at| now.signed_duration_since(last_warning_at) >= cooldown)
        {
            self.last_low_rate_warning_at = None;
            cleared = true;
        }
        if cleared {
            self.cooldown_multiplier = self.cooldown_multiplier.max(1) * 2;
        }
    }
}

impl Default for WarningScheduler {
    fn default() -> Self {
        Self {
            initial_no_kills_sent: false,
            last_no_kills_warning_at: None,
            last_low_rate_warning_at: None,
            cooldown_multiplier: 1,
        }
    }
}

fn emoji_for_event(event_type: &str) -> Option<&'static str> {
    match event_type {
        "cargo_scan" => Some("📦"),
        "ship_scan" => Some("🔎"),
        "security_scan" | "security_attack" => Some("🚨"),
        "kill_bounty" | "kill_bond" => Some("💥"),
        "kill_rate" | "no_kills" => Some("⚠️"),
        "mission_redirected" => Some("✅"),
        "mission_accepted" | "mission_status" | "missions_snapshot" => Some("🎯"),
        "ship_shields" => Some("🛡️"),
        "fighter_hull" | "fighter_destroyed" | "fighter_launch" => Some("🕹️"),
        "ship_hull" => Some("🛠️"),
        "cargo_lost" => Some("🪓"),
        "fuel_report" => Some("⛽"),
        "died" => Some("💀"),
        "main_menu" => Some("🚪"),
        "commander_load" => Some("🔄"),
        "destination_drop" => Some("⚔️"),
        "supercruise_entry" => Some("🚀"),
        "fsd_jump" => Some("☀️"),
        "shipyard_swap" => Some("🚢"),
        "monitor_started" => Some("📖"),
        "monitor_stopped" => Some("📕"),
        "summary" => Some("📝"),
        "shutdown" => Some("🛑"),
        "bait_value_low" => Some("🎣"),
        "merits" => Some("🎫"),
        _ => None,
    }
}

fn bounty_credits(event: &BountyEvent) -> u64 {
    event
        .rewards
        .as_ref()
        .and_then(|rewards| rewards.first())
        .and_then(|reward| reward.reward)
        .or(event.total_reward)
        .unwrap_or(0)
}

fn alert_level(level: u8) -> AlertLevel {
    match level {
        0 | 1 => AlertLevel::Info,
        2 => AlertLevel::Warn,
        _ => AlertLevel::Critical,
    }
}

fn text_contains_any(text: Option<&str>, needles: &[&str]) -> bool {
    let Some(text) = text else {
        return false;
    };
    let lower_text = text.to_ascii_lowercase();
    needles.iter().any(|needle| lower_text.contains(needle))
}

fn clean_text(text: &str) -> String {
    line_safe(text)
}

fn percentage(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 100.0).round() as u8
}

fn push_recent(values: &mut VecDeque<String>, value: String, max: usize) {
    if values.len() == max {
        values.pop_front();
    }
    values.push_back(value);
}

fn combat_rank_name(rank: u8) -> Option<&'static str> {
    COMBAT_RANKS.get(rank as usize).copied()
}

fn display_game_mode(mode: &str) -> String {
    if mode == "Group" {
        "Private Group".to_string()
    } else {
        clean_text(mode)
    }
}

fn is_known_target_ship(ship: &str) -> bool {
    is_easy_ship(ship) || is_hard_ship(ship)
}

fn is_easy_ship(ship: &str) -> bool {
    SHIPS_EASY
        .iter()
        .any(|known| ship.eq_ignore_ascii_case(known))
}

fn is_hard_ship(ship: &str) -> bool {
    SHIPS_HARD
        .iter()
        .any(|known| ship.eq_ignore_ascii_case(known))
}

fn ship_name(raw: Option<&str>, localised: Option<&str>) -> String {
    if let Some(localised) = localised.filter(|value| !value.is_empty()) {
        return clean_text(localised);
    }
    match raw.unwrap_or("[Unknown]").to_ascii_lowercase().as_str() {
        "adder" => "Adder".to_string(),
        "anaconda" => "Anaconda".to_string(),
        "asp" => "Asp Explorer".to_string(),
        "asp_scout" => "Asp Scout".to_string(),
        "cobramkiii" => "Cobra Mk III".to_string(),
        "cobramkiv" => "Cobra Mk IV".to_string(),
        "diamondback" => "Diamondback Scout".to_string(),
        "diamondbackxl" => "Diamondback Explorer".to_string(),
        "eagle" => "Eagle".to_string(),
        "empire_courier" => "Imperial Courier".to_string(),
        "empire_eagle" => "Imperial Eagle".to_string(),
        "federation_gunship" => "Federal Gunship".to_string(),
        "ferdelance" => "Fer-de-Lance".to_string(),
        "krait_mkii" => "Krait Mk II".to_string(),
        "python" => "Python".to_string(),
        "sidewinder" => "Sidewinder".to_string(),
        "type9_military" => "Type-10 Defender".to_string(),
        "typex" | "typex_2" | "typex_3" => "Alliance Combat Ship".to_string(),
        "viper" => "Viper Mk III".to_string(),
        "viper_mkiv" => "Viper Mk IV".to_string(),
        "vulture" => "Vulture".to_string(),
        other => title_identifier(other),
    }
}

fn title_identifier(value: &str) -> String {
    value
        .split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn mission_is_massacre_text(name: Option<&str>, localised: Option<&str>) -> bool {
    let _ = localised;
    text_contains_any(name, &["mission_massacre"])
}

fn text_equals(text: Option<&str>, expected: &str) -> bool {
    text.is_some_and(|text| text.eq_ignore_ascii_case(expected))
}

fn extended_count(enabled: bool, count: u64) -> String {
    if enabled {
        format!(" x{count}")
    } else {
        String::new()
    }
}

fn afk_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds().max(0);
    if total_seconds < 60 {
        return format!("{total_seconds}s");
    }
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        if minutes > 0 {
            format!("{hours}h{minutes}m")
        } else {
            format!("{hours}h")
        }
    } else if seconds > 0 {
        format!("{minutes}m{seconds}s")
    } else {
        format!("{minutes}m")
    }
}

fn compact_number(value: u64) -> String {
    if value >= 999_500 {
        let millions = (value as f64 / 1_000_000.0 * 10.0).round() / 10.0;
        if millions.fract() == 0.0 {
            format!("{}m", millions as u64)
        } else {
            format!("{millions:.1}m")
        }
    } else if value >= 1_000 {
        format!("{}k", (value as f64 / 1_000.0).round() as u64)
    } else {
        value.to_string()
    }
}

fn compact_decimal(value: f64) -> String {
    if value.fract() == 0.0 {
        (value as u64).to_string()
    } else {
        format!("{value:.1}")
    }
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars + 1 {
        return value.to_string();
    }
    let mut truncated = value.chars().take(max_chars).collect::<String>();
    while truncated.ends_with(' ') {
        truncated.pop();
    }
    format!("{truncated}…")
}
