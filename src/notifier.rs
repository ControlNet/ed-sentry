use chrono::{DateTime, Utc};

use crate::config::{LogLevelConfig, MonitorConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warn,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub event_type: String,
    pub level: u8,
    pub alert_level: AlertLevel,
    pub emoji: Option<String>,
    pub terminal_text: String,
    pub remote_text: String,
    pub timestamp: DateTime<Utc>,
    pub mention: bool,
}

impl Notification {
    pub fn new(
        event_type: impl Into<String>,
        level: u8,
        alert_level: AlertLevel,
        emoji: Option<String>,
        terminal_text: impl Into<String>,
        remote_text: impl Into<String>,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            event_type: event_type.into(),
            level,
            alert_level,
            emoji,
            terminal_text: terminal_text.into(),
            remote_text: remote_text.into(),
            timestamp,
            mention: level >= 2,
        }
    }
}

pub trait Notifier {
    fn send(&mut self, notification: &Notification) -> anyhow::Result<()>;
}

#[derive(Clone, Debug, Default)]
pub struct FakeNotifier {
    notifications: Vec<Notification>,
}

impl FakeNotifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn notifications(&self) -> &[Notification] {
        &self.notifications
    }

    pub fn into_notifications(self) -> Vec<Notification> {
        self.notifications
    }

    pub fn clear(&mut self) {
        self.notifications.clear();
    }
}

impl Notifier for FakeNotifier {
    fn send(&mut self, notification: &Notification) -> anyhow::Result<()> {
        self.notifications.push(notification.clone());
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct NotificationDispatcher<N> {
    notifier: N,
}

impl<N: Notifier> NotificationDispatcher<N> {
    pub fn new(notifier: N, _duplicate_max: u16) -> Self {
        Self { notifier }
    }

    pub fn from_config(
        notifier: N,
        _monitor: &MonitorConfig,
        _log_levels: &LogLevelConfig,
    ) -> Self {
        Self { notifier }
    }

    pub fn dispatch(&mut self, notification: Notification) -> anyhow::Result<()> {
        if notification.level == 0 {
            return Ok(());
        }

        self.notifier.send(&notification)
    }

    pub fn notifier(&self) -> &N {
        &self.notifier
    }

    pub fn notifier_mut(&mut self) -> &mut N {
        &mut self.notifier
    }

    pub fn into_notifier(self) -> N {
        self.notifier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-06-09T15:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn notification(event_type: &str, level: u8, terminal_text: &str) -> Notification {
        Notification::new(
            event_type,
            level,
            AlertLevel::Info,
            None,
            terminal_text,
            terminal_text,
            test_timestamp(),
        )
    }

    #[test]
    fn notifier_fake_notifier_records_sent_notifications() {
        let mut notifier = FakeNotifier::new();
        let notification = notification("Kill", 1, "Bounty confirmed");

        notifier.send(&notification).unwrap();

        assert_eq!(notifier.notifications(), &[notification]);
    }

    #[test]
    fn notifier_level_routing_ignores_zero_and_marks_mentions_at_two() {
        let mut dispatcher = NotificationDispatcher::new(FakeNotifier::new(), 5);

        dispatcher
            .dispatch(notification("SummaryFaction", 0, "Faction summary"))
            .unwrap();
        dispatcher
            .dispatch(notification("FuelReport", 1, "Fuel stable"))
            .unwrap();
        dispatcher
            .dispatch(notification("FighterHull", 2, "Fighter hull warning"))
            .unwrap();
        dispatcher
            .dispatch(notification("ShipHull", 3, "Ship hull critical"))
            .unwrap();

        let notifications = dispatcher.notifier().notifications();
        assert_eq!(notifications.len(), 3);
        assert_eq!(notifications[0].level, 1);
        assert_eq!(notifications[1].level, 2);
        assert_eq!(notifications[2].level, 3);
        assert!(!notifications[0].mention);
        assert!(notifications[1].mention);
        assert!(notifications[2].mention);
    }

    #[test]
    fn notifier_level_two_is_mention_capable() {
        let level_one = notification("FuelReport", 1, "Fuel stable");
        let level_two = notification("ShipHull", 2, "Ship hull critical");
        let level_three = notification("Death", 3, "Ship destroyed");

        assert!(!level_one.mention);
        assert!(level_two.mention);
        assert!(level_three.mention);
    }

    #[test]
    fn notifier_terminal_duplicates_are_not_suppressed() {
        let mut dispatcher = NotificationDispatcher::new(FakeNotifier::new(), 5);

        for _ in 0..8 {
            dispatcher
                .dispatch(notification("Bounty", 1, "Bounty voucher received"))
                .unwrap();
        }

        let notifications = dispatcher.notifier().notifications();
        assert_eq!(notifications.len(), 8);
        assert!(notifications
            .iter()
            .all(|notification| notification.event_type == "Bounty"));
    }

    #[test]
    fn notifier_dispatcher_ignores_duplicate_settings_for_terminal_delivery() {
        let monitor = MonitorConfig {
            duplicate_max: 1,
            ..MonitorConfig::default()
        };
        let log_levels = LogLevelConfig::default();
        let mut dispatcher =
            NotificationDispatcher::from_config(FakeNotifier::new(), &monitor, &log_levels);

        dispatcher
            .dispatch(notification("Fuel", 1, "Fuel low"))
            .unwrap();
        dispatcher
            .dispatch(notification("Fuel", 1, "Fuel low"))
            .unwrap();
        dispatcher
            .dispatch(notification("Fuel", 1, "Fuel low"))
            .unwrap();

        let notifications = dispatcher.notifier().notifications();
        assert_eq!(notifications.len(), 3);
        assert!(notifications
            .iter()
            .all(|notification| notification.event_type == "Fuel"));
    }
}
