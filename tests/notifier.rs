use chrono::{DateTime, Utc};
use ed_afk_monitor::notifier::{AlertLevel, FakeNotifier, Notification, NotificationDispatcher};

#[test]
fn notifier_public_api_driver_dispatches_through_fake_notifier() {
    let timestamp = DateTime::parse_from_rfc3339("2026-06-09T15:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let mut dispatcher = NotificationDispatcher::new(FakeNotifier::new(), 5);

    dispatcher
        .dispatch(Notification::new(
            "DriverEvent",
            3,
            AlertLevel::Critical,
            Some("!".to_string()),
            "Driver notification",
            "Driver remote notification",
            timestamp,
        ))
        .unwrap();

    let notifications = dispatcher.notifier().notifications();
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].event_type, "DriverEvent");
    assert!(notifications[0].mention);
}
