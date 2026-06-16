use std::io::Write;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::notifier::{Notification, Notifier};
use crate::terminal::TerminalNotifier;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusCadence {
    interval: Duration,
    last_published_at: Option<DateTime<Utc>>,
}

impl StatusCadence {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_published_at: None,
        }
    }

    pub fn from_interval_seconds(seconds: u64) -> Self {
        Self::new(Duration::from_secs(seconds))
    }

    pub fn is_due(&self, now: DateTime<Utc>, force: bool) -> bool {
        if force || self.last_published_at.is_none() {
            return true;
        }

        let last_published_at = self.last_published_at.expect("checked above");
        now.signed_duration_since(last_published_at)
            .to_std()
            .is_ok_and(|elapsed| elapsed >= self.interval)
    }

    pub fn should_publish(&mut self, now: DateTime<Utc>, force: bool) -> bool {
        if !self.is_due(now, force) {
            return false;
        }

        self.last_published_at = Some(now);
        true
    }
}

impl Default for StatusCadence {
    fn default() -> Self {
        Self::from_interval_seconds(60)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliveryWarning {
    pub message: String,
}

#[async_trait]
pub trait RemoteDelivery: Send {
    async fn send(&mut self, notification: &Notification) -> anyhow::Result<()>;

    async fn publish_status(
        &mut self,
        status: &str,
        now: DateTime<Utc>,
        force: bool,
    ) -> anyhow::Result<()>;
}

pub struct DeliveryHub<W: Write> {
    terminal: TerminalNotifier<W>,
    matrix: Option<Box<dyn RemoteDelivery>>,
    status_cadence: StatusCadence,
}

impl<W: Write> DeliveryHub<W> {
    pub fn new(terminal: TerminalNotifier<W>, matrix: Option<Box<dyn RemoteDelivery>>) -> Self {
        Self {
            terminal,
            matrix,
            status_cadence: StatusCadence::default(),
        }
    }

    pub fn terminal_only(terminal: TerminalNotifier<W>) -> Self {
        Self::new(terminal, None)
    }

    pub fn with_status_cadence(mut self, status_cadence: StatusCadence) -> Self {
        self.status_cadence = status_cadence;
        self
    }

    pub fn supports_status_line(&self) -> bool {
        self.terminal.supports_status_line()
    }

    pub fn into_terminal(self) -> TerminalNotifier<W> {
        self.terminal
    }

    pub async fn send_notifications(
        &mut self,
        notifications: &[Notification],
    ) -> anyhow::Result<Vec<DeliveryWarning>> {
        let mut warnings = Vec::new();
        for notification in notifications {
            if notification.level == 0 {
                continue;
            }

            self.terminal.send(notification)?;
            if let Some(matrix) = self.matrix.as_mut() {
                if let Err(error) = matrix.send(notification).await {
                    warnings.push(DeliveryWarning {
                        message: error.to_string(),
                    });
                }
            }
        }
        Ok(warnings)
    }

    pub fn send_terminal_notifications(
        &mut self,
        notifications: &[Notification],
    ) -> anyhow::Result<()> {
        for notification in notifications {
            if notification.level == 0 {
                continue;
            }

            self.terminal.send(notification)?;
        }
        Ok(())
    }

    pub async fn send_remote_notifications(
        &mut self,
        notifications: &[Notification],
    ) -> anyhow::Result<Vec<DeliveryWarning>> {
        let mut warnings = Vec::new();
        let Some(matrix) = self.matrix.as_mut() else {
            return Ok(warnings);
        };

        for notification in notifications {
            if notification.level == 0 {
                continue;
            }

            if let Err(error) = matrix.send(notification).await {
                warnings.push(DeliveryWarning {
                    message: error.to_string(),
                });
            }
        }
        Ok(warnings)
    }

    pub async fn publish_status(
        &mut self,
        status: &str,
        now: DateTime<Utc>,
        force: bool,
        render_terminal: bool,
    ) -> anyhow::Result<Vec<DeliveryWarning>> {
        if render_terminal {
            self.terminal.render_status(status)?;
        }

        let mut warnings = Vec::new();
        if let Some(matrix) = self.matrix.as_mut() {
            if !self.status_cadence.should_publish(now, force) {
                return Ok(warnings);
            }

            if let Err(error) = matrix.publish_status(status, now, force).await {
                warnings.push(DeliveryWarning {
                    message: error.to_string(),
                });
            }
        }
        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifier::AlertLevel;
    use crate::terminal::TerminalNotifier;
    use crate::time::TimeDisplayZone;
    use std::sync::{Arc, Mutex};

    fn timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2035-06-09T16:30:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn notification(level: u8, text: &str) -> Notification {
        Notification::new(
            "delivery_fixture",
            level,
            AlertLevel::Info,
            None,
            text,
            text,
            timestamp(),
        )
    }

    type RecordedStatuses = Arc<Mutex<Vec<(String, DateTime<Utc>, bool)>>>;

    struct FailingRemote;

    struct RecordingRemote {
        statuses: RecordedStatuses,
    }

    #[async_trait]
    impl RemoteDelivery for FailingRemote {
        async fn send(&mut self, _notification: &Notification) -> anyhow::Result<()> {
            anyhow::bail!("remote send failed")
        }

        async fn publish_status(
            &mut self,
            _status: &str,
            _now: DateTime<Utc>,
            _force: bool,
        ) -> anyhow::Result<()> {
            anyhow::bail!("remote status failed")
        }
    }

    #[async_trait]
    impl RemoteDelivery for RecordingRemote {
        async fn send(&mut self, _notification: &Notification) -> anyhow::Result<()> {
            Ok(())
        }

        async fn publish_status(
            &mut self,
            status: &str,
            now: DateTime<Utc>,
            force: bool,
        ) -> anyhow::Result<()> {
            self.statuses
                .lock()
                .unwrap()
                .push((status.to_string(), now, force));
            Ok(())
        }
    }

    #[test]
    fn status_cadence_respects_interval() {
        let start = timestamp();
        let mut cadence = StatusCadence::new(Duration::from_secs(60));

        assert!(cadence.is_due(start, false));
        assert!(cadence.should_publish(start, false));
        assert!(!cadence.is_due(start + chrono::Duration::seconds(59), false));
        assert!(!cadence.should_publish(start + chrono::Duration::seconds(59), false));
        assert!(cadence.is_due(start + chrono::Duration::seconds(60), false));
        assert!(cadence.should_publish(start + chrono::Duration::seconds(60), false));
        assert!(!cadence.is_due(start + chrono::Duration::seconds(61), false));
        assert!(cadence.is_due(start + chrono::Duration::seconds(61), true));
        assert!(cadence.should_publish(start + chrono::Duration::seconds(61), true));
        assert!(cadence.is_due(start + chrono::Duration::seconds(62), true));
    }

    #[tokio::test]
    async fn delivery_hub_filters_level_zero_and_returns_remote_warnings() {
        let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
        let mut hub = DeliveryHub::new(terminal, Some(Box::new(FailingRemote)));

        let warnings = hub
            .send_notifications(&[notification(0, "Hidden"), notification(1, "Visible")])
            .await
            .unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "remote send failed");
        let output = String::from_utf8(hub.into_terminal().into_inner()).unwrap();
        assert!(!output.contains("Hidden"), "{output}");
        assert!(output.contains("Visible"), "{output}");
    }

    #[tokio::test]
    async fn delivery_hub_renders_status_before_returning_remote_warning() {
        let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
        let mut hub = DeliveryHub::new(terminal, Some(Box::new(FailingRemote)));

        let warnings = hub
            .publish_status("Kills 71", timestamp(), true, true)
            .await
            .unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "remote status failed");
        let output = String::from_utf8(hub.into_terminal().into_inner()).unwrap();
        assert!(output.contains("Kills 71"), "{output}");
    }

    #[tokio::test]
    async fn delivery_hub_renders_every_status_but_gates_remote_by_cadence() {
        let start = timestamp();
        let statuses = Arc::new(Mutex::new(Vec::new()));
        let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
        let mut hub = DeliveryHub::new(
            terminal,
            Some(Box::new(RecordingRemote {
                statuses: Arc::clone(&statuses),
            })),
        )
        .with_status_cadence(StatusCadence::new(Duration::from_secs(60)));

        hub.publish_status("First", start, false, true)
            .await
            .unwrap();
        hub.publish_status("Second", start + chrono::Duration::seconds(30), false, true)
            .await
            .unwrap();
        hub.publish_status("Third", start + chrono::Duration::seconds(60), false, true)
            .await
            .unwrap();
        hub.publish_status("Forced", start + chrono::Duration::seconds(61), true, true)
            .await
            .unwrap();

        let output = String::from_utf8(hub.into_terminal().into_inner()).unwrap();
        assert!(output.contains("First"), "{output}");
        assert!(output.contains("Second"), "{output}");
        assert!(output.contains("Third"), "{output}");
        assert!(output.contains("Forced"), "{output}");
        let statuses = statuses.lock().unwrap();
        assert_eq!(statuses.len(), 3);
        assert_eq!(statuses[0].0, "First");
        assert_eq!(statuses[1].0, "Third");
        assert_eq!(statuses[2].0, "Forced");
        assert!(statuses[2].2);
    }

    #[tokio::test]
    async fn delivery_hub_can_publish_remote_status_without_terminal_render() {
        let statuses = Arc::new(Mutex::new(Vec::new()));
        let terminal = TerminalNotifier::plain(Vec::new(), TimeDisplayZone::Utc);
        let mut hub = DeliveryHub::new(
            terminal,
            Some(Box::new(RecordingRemote {
                statuses: Arc::clone(&statuses),
            })),
        );

        hub.publish_status("Remote only", timestamp(), true, false)
            .await
            .unwrap();

        let output = String::from_utf8(hub.into_terminal().into_inner()).unwrap();
        assert!(output.is_empty(), "{output}");
        let statuses = statuses.lock().unwrap();
        assert_eq!(statuses.len(), 1);
        assert_eq!(statuses[0].0, "Remote only");
        assert!(statuses[0].2);
    }
}
