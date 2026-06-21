use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::app::{AppEventStore, AppSnapshot, MatrixStartupStatus, WebStartupStatus};
use crate::config::RuntimeConfig;
use crate::event::{parse_journal_line, JournalEvent};
use crate::journal::{live_poll_interval, preload_journal_file, LiveTail, PreloadRecord};
use crate::mission::MissionTracker;
use crate::monitor::EventMonitor;
use crate::notifier::{AlertLevel, Notification};
use crate::text::line_safe;

use super::paths::{journal_basename, startup_commander};
use super::types::{
    JournalSelector, MonitorStartup, RuntimeBatch, RuntimeError, RuntimeNotification,
    RuntimeNotificationDelivery, RuntimeStatusSnapshot,
};

mod snapshot;

pub struct MonitorRuntime {
    config: RuntimeConfig,
    startup: MonitorStartup,
    tail: LiveTail,
    monitor: EventMonitor,
    missions: MissionTracker,
    preload_records: Vec<PreloadRecord<JournalEvent>>,
    matrix_status: MatrixStartupStatus,
    pub(super) web_status: WebStartupStatus,
    events: AppEventStore,
}

impl MonitorRuntime {
    pub fn start(
        config: &RuntimeConfig,
        selector: &mut impl JournalSelector,
        matrix_status: MatrixStartupStatus,
        web_status: WebStartupStatus,
    ) -> Result<Self, RuntimeError> {
        let journal_file = selector.select(config)?;
        let preload = preload_journal_file(&journal_file, parse_journal_line)?;
        let startup = MonitorStartup {
            journal_file: journal_file.clone(),
            commander: startup_commander(&preload.records).map(|name| line_safe(&name)),
            preload_line_count: preload.records.len(),
            preload_eof_offset: preload.eof_offset,
        };
        let tail = LiveTail::from_preload(&journal_file, &preload);
        let monitor = EventMonitor::from_runtime_config(config);
        let missions = MissionTracker::new();
        let events = AppEventStore::from_state(
            monitor.state(),
            &missions,
            Utc::now(),
            matrix_status.clone(),
            web_status.clone(),
        );

        Ok(Self {
            config: config.clone(),
            startup,
            tail,
            monitor,
            missions,
            preload_records: preload.records,
            matrix_status,
            web_status,
            events,
        })
    }

    pub fn startup(&self) -> &MonitorStartup {
        &self.startup
    }

    pub fn set_matrix_status(&mut self, status: MatrixStartupStatus) {
        self.matrix_status = status;
    }

    pub fn event_store(&self) -> AppEventStore {
        self.events.clone()
    }

    pub fn process_preload(&mut self, program_started_at: DateTime<Utc>) -> RuntimeBatch {
        let mut batch = self.empty_batch(program_started_at);
        let records = std::mem::take(&mut self.preload_records);
        for record in records {
            match record.result {
                Ok(event) => {
                    let delivery = if event.timestamp() < program_started_at {
                        RuntimeNotificationDelivery::TerminalOnly
                    } else {
                        RuntimeNotificationDelivery::All
                    };
                    self.process_event(&event, delivery, &mut batch);
                }
                Err(error) => self.push_warning(
                    &mut batch,
                    format!(
                        "Malformed journal line {} during preload: {}",
                        record.line_number, error.message
                    ),
                    program_started_at,
                ),
            }
        }
        batch.snapshot = self.snapshot(program_started_at);
        self.events.publish_snapshot(batch.snapshot.clone());
        batch
    }

    pub fn reset_session(&mut self, now: DateTime<Utc>) -> RuntimeBatch {
        self.monitor.state_mut().reset_session_counters();
        let notification = Notification::new(
            "session_reset",
            1,
            AlertLevel::Info,
            Some("🔄".to_string()),
            "Session stats reset",
            "Session stats reset",
            now,
        );
        self.batch_from_notifications([notification], RuntimeNotificationDelivery::All, now)
    }

    pub fn start_monitor_if_preloaded(&mut self, now: DateTime<Utc>) -> RuntimeBatch {
        if self.startup.preload_line_count == 0 {
            return self.empty_batch(now);
        }
        let journal_file = journal_basename(&self.startup.journal_file);
        let summary = format!("Started monitoring {journal_file}");
        self.events
            .record_lifecycle("monitor_started", summary, now);
        let notification = self.monitor.start_monitor(&journal_file, now);
        self.batch_from_notifications([notification], RuntimeNotificationDelivery::All, now)
    }

    pub fn poll_once(&mut self, now: DateTime<Utc>) -> Result<RuntimeBatch, RuntimeError> {
        let mut batch = self.empty_batch(now);
        let poll = self.tail.poll(parse_journal_line)?;
        for warning in poll.warnings {
            self.push_warning(&mut batch, warning.to_string(), now);
        }
        for record in poll.records {
            match record.result {
                Ok(event) => {
                    self.process_event(&event, RuntimeNotificationDelivery::All, &mut batch)
                }
                Err(error) => self.push_warning(
                    &mut batch,
                    format!(
                        "Malformed journal line at byte offset {}: {}",
                        record.start_offset, error.message
                    ),
                    now,
                ),
            }
        }
        let notifications = self.monitor.check_warnings_at(now, false);
        self.extend_notifications(notifications, RuntimeNotificationDelivery::All, &mut batch);
        batch.snapshot = self.snapshot(now);
        self.events.publish_snapshot(batch.snapshot.clone());
        Ok(batch)
    }

    pub fn status_snapshot(
        &mut self,
        now: DateTime<Utc>,
        force_publish: bool,
    ) -> RuntimeStatusSnapshot {
        let status_line = self
            .config
            .monitor
            .live_status
            .then(|| self.monitor.render_status_line(now));
        let dynamic_title = self
            .config
            .monitor
            .dynamic_title
            .then(|| self.monitor.render_dynamic_title(now));
        if let Some(summary) = status_line.as_deref().or(dynamic_title.as_deref()) {
            self.events
                .record_status_update("runtime_status", summary, now);
        }
        let snapshot = self.snapshot(now);
        self.events.publish_snapshot(snapshot.clone());
        RuntimeStatusSnapshot {
            status_line,
            dynamic_title,
            force_publish,
            snapshot,
        }
    }

    pub fn poll_interval(&self) -> Duration {
        live_poll_interval(&self.config)
    }

    pub fn snapshot(&self, now: DateTime<Utc>) -> AppSnapshot {
        snapshot::runtime_snapshot(self, now)
    }

    fn process_event(
        &mut self,
        event: &JournalEvent,
        delivery: RuntimeNotificationDelivery,
        batch: &mut RuntimeBatch,
    ) {
        self.missions.apply_event(event);
        let notifications = self.monitor.process_event(event);
        self.extend_notifications(notifications, delivery, batch);
    }

    fn batch_from_notifications(
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

    fn extend_notifications(
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

    fn push_warning(&self, batch: &mut RuntimeBatch, warning: String, now: DateTime<Utc>) {
        let warning = line_safe(&warning);
        self.events.record_warning(&warning, now);
        batch.warnings.push(warning);
    }

    fn empty_batch(&self, now: DateTime<Utc>) -> RuntimeBatch {
        RuntimeBatch::empty(self.snapshot(now))
    }
}
