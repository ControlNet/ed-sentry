use chrono::{DateTime, Duration, TimeZone, Utc};
use ed_sentry::config::{LogLevelConfig, MonitorConfig};
use ed_sentry::event::{BountyEvent, BountyReward, JournalEvent, SupercruiseDestinationDropEvent};
use ed_sentry::monitor::EventMonitor;
use ed_sentry::notifier::Notification;

fn timestamp(minutes: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2035, 4, 12, 8, 0, 0).single().unwrap() + Duration::minutes(minutes)
}

fn monitor_config() -> MonitorConfig {
    MonitorConfig {
        warn_kill_rate: 15,
        warn_kill_rate_delay_minutes: 5,
        warn_no_kills_initial_minutes: 5,
        warn_no_kills_minutes: 10,
        warn_cooldown_minutes: 30,
        ..MonitorConfig::default()
    }
}

fn res_drop(minutes: i64) -> JournalEvent {
    JournalEvent::SupercruiseDestinationDrop(SupercruiseDestinationDropEvent {
        timestamp: timestamp(minutes),
        event: "SupercruiseDestinationDrop".to_string(),
        raw: None,
        destination_type: Some("ResourceExtraction".to_string()),
        destination_type_localised: Some("Resource Extraction Site".to_string()),
    })
}

fn bounty(minutes: i64) -> JournalEvent {
    JournalEvent::Bounty(BountyEvent {
        timestamp: timestamp(minutes),
        event: "Bounty".to_string(),
        raw: None,
        total_reward: Some(4_200),
        rewards: Some(vec![BountyReward {
            faction: Some("Fixture Security".to_string()),
            reward: Some(4_200),
        }]),
        victim_faction: Some("Fixture Raiders".to_string()),
        victim_faction_localised: None,
        target: Some("viper".to_string()),
        target_localised: None,
        pilot_name_localised: None,
    })
}

fn monitor(config: MonitorConfig, log_levels: LogLevelConfig) -> EventMonitor {
    EventMonitor::new(config, log_levels)
}

fn push_event(
    notifications: &mut Vec<Notification>,
    monitor: &mut EventMonitor,
    event: &JournalEvent,
) {
    notifications.extend(monitor.process_event(event));
}

fn push_warning(
    notifications: &mut Vec<Notification>,
    monitor: &mut EventMonitor,
    minutes: i64,
    preload: bool,
) {
    notifications.extend(monitor.check_warnings_at(timestamp(minutes), preload));
}

fn notifications<'a>(notifications: &'a [Notification], event_type: &str) -> Vec<&'a Notification> {
    notifications
        .iter()
        .filter(|notification| notification.event_type == event_type)
        .collect()
}

#[test]
fn warnings_no_kill_threshold_initial_warning_fires_once() {
    let mut monitor = monitor(monitor_config(), LogLevelConfig::default());
    let mut sent = Vec::new();

    push_event(&mut sent, &mut monitor, &res_drop(0));
    push_warning(&mut sent, &mut monitor, 4, false);
    push_warning(&mut sent, &mut monitor, 5, false);
    push_warning(&mut sent, &mut monitor, 35, false);

    let warnings = notifications(&sent, "no_kills");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].timestamp, timestamp(5));
    assert_eq!(warnings[0].level, LogLevelConfig::default().no_kills);
    assert_eq!(warnings[0].terminal_text, "No kills logged for 5 minutes");
}

#[test]
fn warnings_later_no_kill_threshold_and_cooldown() {
    let config = MonitorConfig {
        warn_kill_rate: 0,
        ..monitor_config()
    };
    let mut monitor = monitor(config, LogLevelConfig::default());
    let mut sent = Vec::new();

    push_event(&mut sent, &mut monitor, &res_drop(0));
    push_event(&mut sent, &mut monitor, &bounty(1));
    push_warning(&mut sent, &mut monitor, 10, false);
    push_warning(&mut sent, &mut monitor, 11, false);
    push_warning(&mut sent, &mut monitor, 40, false);
    push_warning(&mut sent, &mut monitor, 41, false);
    let warnings = notifications(&sent, "no_kills");
    assert_eq!(warnings.len(), 2);
    assert_eq!(warnings[0].timestamp, timestamp(11));
    assert_eq!(
        warnings[0].terminal_text,
        "Last logged kill was 10 minutes ago"
    );
    assert_eq!(warnings[1].timestamp, timestamp(41));
}

#[test]
fn warnings_new_kill_resets_no_kill_timer() {
    let config = MonitorConfig {
        warn_kill_rate: 0,
        ..monitor_config()
    };
    let mut monitor = monitor(config, LogLevelConfig::default());
    let mut sent = Vec::new();

    push_event(&mut sent, &mut monitor, &res_drop(0));
    push_event(&mut sent, &mut monitor, &bounty(8));
    push_warning(&mut sent, &mut monitor, 13, false);
    push_warning(&mut sent, &mut monitor, 18, false);

    let warnings = notifications(&sent, "no_kills");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].timestamp, timestamp(18));
}

#[test]
fn warnings_low_kill_rate_threshold_cooldown_and_kill_reset() {
    let config = MonitorConfig {
        warn_no_kills_minutes: 120,
        ..monitor_config()
    };
    let mut monitor = monitor(config, LogLevelConfig::default());
    let mut sent = Vec::new();

    push_event(&mut sent, &mut monitor, &res_drop(0));
    push_event(&mut sent, &mut monitor, &bounty(1));
    push_warning(&mut sent, &mut monitor, 4, false);
    push_warning(&mut sent, &mut monitor, 6, false);
    push_warning(&mut sent, &mut monitor, 20, false);
    push_event(&mut sent, &mut monitor, &bounty(21));
    push_warning(&mut sent, &mut monitor, 22, false);
    push_warning(&mut sent, &mut monitor, 36, false);

    let warnings = notifications(&sent, "kill_rate");
    assert_eq!(warnings.len(), 2);
    assert_eq!(warnings[0].timestamp, timestamp(6));
    assert_eq!(warnings[1].timestamp, timestamp(36));
    assert!(warnings[0].terminal_text.contains("Kill rate of"));
    assert!(warnings[0].terminal_text.contains("threshold"));
}

#[test]
fn warnings_low_kill_rate_suppresses_later_no_kills_when_both_apply() {
    let mut monitor = monitor(monitor_config(), LogLevelConfig::default());
    let mut sent = Vec::new();

    push_event(&mut sent, &mut monitor, &res_drop(0));
    push_event(&mut sent, &mut monitor, &bounty(1));
    push_warning(&mut sent, &mut monitor, 20, false);

    let kill_rate = notifications(&sent, "kill_rate");
    assert_eq!(kill_rate.len(), 1);
    assert_eq!(kill_rate[0].timestamp, timestamp(20));
    assert!(kill_rate[0].terminal_text.contains("Kill rate of"));
    assert!(notifications(&sent, "no_kills").is_empty());
}

#[test]
fn warnings_disabled_during_preload() {
    let mut monitor = monitor(monitor_config(), LogLevelConfig::default());
    let mut sent = Vec::new();

    push_event(&mut sent, &mut monitor, &res_drop(0));
    push_warning(&mut sent, &mut monitor, 60, true);

    assert!(monitor.state().active_session);
    assert!(notifications(&sent, "no_kills").is_empty());
    assert!(notifications(&sent, "kill_rate").is_empty());
}

#[test]
fn warnings_do_not_fire_before_session_start() {
    let mut monitor = monitor(monitor_config(), LogLevelConfig::default());
    let mut sent = Vec::new();

    push_warning(&mut sent, &mut monitor, 60, false);

    assert!(sent.is_empty());
}

#[test]
fn warnings_level_zero_suppresses_delivery_without_breaking_state() {
    let log_levels = LogLevelConfig {
        no_kills: 0,
        kill_rate: 0,
        ..LogLevelConfig::default()
    };
    let mut monitor = monitor(monitor_config(), log_levels);
    let mut sent = Vec::new();

    push_event(&mut sent, &mut monitor, &res_drop(0));
    push_warning(&mut sent, &mut monitor, 5, false);

    assert!(monitor.state().active_session);
    assert_eq!(monitor.state().kills, 0);
    assert_eq!(notifications(&sent, "no_kills")[0].level, 0);
    assert!(notifications(&sent, "kill_rate").is_empty());
}
