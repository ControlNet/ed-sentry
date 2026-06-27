use chrono::{DateTime, Utc};

use crate::notifier::Notification;
use crate::text::line_safe;

use super::MonitorRuntime;
use crate::app::runtime::types::{RuntimeBatch, RuntimeNotification, RuntimeNotificationDelivery};

impl MonitorRuntime {
    pub(super) fn batch_from_notifications(
        &mut self,
        notifications: impl IntoIterator<Item = Notification>,
        delivery: RuntimeNotificationDelivery,
        now: DateTime<Utc>,
    ) -> RuntimeBatch {
        let mut batch = self.empty_batch(now);
        self.extend_notifications(notifications, delivery, &mut batch);
        batch.snapshot = self.snapshot(now);
        self.events.publish_snapshot(batch.snapshot.clone());
        batch
    }

    pub(super) fn extend_notifications(
        &mut self,
        notifications: impl IntoIterator<Item = Notification>,
        delivery: RuntimeNotificationDelivery,
        batch: &mut RuntimeBatch,
    ) {
        for notification in notifications {
            if let Some(stored) = self.events.record_notification(&notification) {
                batch.notifications.push(RuntimeNotification {
                    notification,
                    view: stored.view,
                    feed_item: stored.feed_item,
                    delivery,
                });
            }
        }
    }

    pub(super) fn push_warning(
        &self,
        batch: &mut RuntimeBatch,
        warning: String,
        now: DateTime<Utc>,
    ) {
        let warning = line_safe(&warning);
        self.events.record_warning(&warning, now);
        batch.warnings.push(warning);
    }

    pub(super) fn empty_batch(&self, now: DateTime<Utc>) -> RuntimeBatch {
        RuntimeBatch::empty(self.snapshot(now))
    }
}
