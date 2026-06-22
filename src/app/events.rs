use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::mission::MissionTracker;
use crate::notifier::Notification;
use crate::state::SessionState;
use crate::text::line_safe;

use super::display::display_timestamp;
use super::{AppSnapshot, EventFeedItem, MatrixStartupStatus, NotificationView, WebStartupStatus};

pub const DEFAULT_RECENT_EVENT_CAPACITY: usize = 200;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AppEventBootstrap {
    pub snapshot: AppSnapshot,
    pub recent_events: Vec<EventFeedItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppLiveUpdate {
    Snapshot { snapshot: Box<AppSnapshot> },
    Event { item: EventFeedItem },
}

#[derive(Debug)]
pub struct AppEventSubscriber {
    pub bootstrap: AppEventBootstrap,
    pub live: Receiver<AppLiveUpdate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoredNotification {
    pub view: NotificationView,
    pub feed_item: EventFeedItem,
}

#[derive(Clone, Debug)]
pub struct AppEventStore {
    inner: Arc<Mutex<AppEventStoreState>>,
}

#[derive(Debug)]
struct AppEventStoreState {
    capacity: usize,
    snapshot: AppSnapshot,
    notifications: VecDeque<NotificationView>,
    events: VecDeque<EventFeedItem>,
    subscribers: Vec<Sender<AppLiveUpdate>>,
    next_sequence: u64,
}

enum SystemEventKind {
    Lifecycle,
    Warning,
}

impl AppEventStore {
    pub fn with_capacity(snapshot: AppSnapshot, capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppEventStoreState {
                capacity,
                snapshot,
                notifications: VecDeque::with_capacity(capacity),
                events: VecDeque::with_capacity(capacity),
                subscribers: Vec::new(),
                next_sequence: 0,
            })),
        }
    }

    pub fn new(snapshot: AppSnapshot) -> Self {
        Self::with_capacity(snapshot, DEFAULT_RECENT_EVENT_CAPACITY)
    }

    pub fn from_state(
        state: &SessionState,
        missions: &MissionTracker,
        now: DateTime<Utc>,
        matrix: MatrixStartupStatus,
        web: WebStartupStatus,
    ) -> Self {
        Self::new(AppSnapshot::from_state(state, missions, now, matrix, web))
    }

    pub fn subscribe(&self) -> AppEventSubscriber {
        let (sender, live) = mpsc::channel();
        let mut state = self.lock_state();
        let bootstrap = state.bootstrap();
        state.subscribers.push(sender);
        AppEventSubscriber { bootstrap, live }
    }

    pub fn publish_snapshot(&self, snapshot: AppSnapshot) {
        let mut state = self.lock_state();
        state.snapshot = state.snapshot_with_history(snapshot);
        let snapshot = state.snapshot.clone();
        state.broadcast(AppLiveUpdate::Snapshot {
            snapshot: Box::new(snapshot),
        });
    }

    pub fn snapshot_with_history(&self, snapshot: AppSnapshot) -> AppSnapshot {
        self.lock_state().snapshot_with_history(snapshot)
    }

    pub fn record_notification(&self, notification: &Notification) -> Option<StoredNotification> {
        if notification.level == 0 {
            return None;
        }
        let view = NotificationView::from(notification);
        let feed_item = EventFeedItem::from(notification);
        self.record_notification_views(view, feed_item)
    }

    pub fn record_lifecycle(
        &self,
        event_type: impl AsRef<str>,
        summary: impl AsRef<str>,
        timestamp: DateTime<Utc>,
    ) {
        let item = self.system_event(
            SystemEventKind::Lifecycle,
            event_type.as_ref(),
            summary.as_ref(),
            timestamp,
        );
        self.record_event(item);
    }

    pub fn record_warning(&self, summary: impl AsRef<str>, timestamp: DateTime<Utc>) {
        let item = self.system_event(
            SystemEventKind::Warning,
            "runtime_warning",
            summary.as_ref(),
            timestamp,
        );
        self.record_event(item);
    }

    fn record_notification_views(
        &self,
        view: NotificationView,
        feed_item: EventFeedItem,
    ) -> Option<StoredNotification> {
        let stored = StoredNotification {
            view: view.clone(),
            feed_item: feed_item.clone(),
        };
        let mut state = self.lock_state();
        state.push_notification(view);
        state.push_event(feed_item.clone());
        state.broadcast(AppLiveUpdate::Event { item: feed_item });
        Some(stored)
    }

    fn record_event(&self, item: EventFeedItem) {
        let mut state = self.lock_state();
        state.push_event(item.clone());
        state.broadcast(AppLiveUpdate::Event { item });
    }

    fn system_event(
        &self,
        kind: SystemEventKind,
        event_type: &str,
        summary: &str,
        timestamp: DateTime<Utc>,
    ) -> EventFeedItem {
        let mut state = self.lock_state();
        let sequence = state.next_sequence();
        drop(state);

        let source = kind.source();
        let event_type = line_safe(event_type);
        EventFeedItem {
            id: format!(
                "{}:{}:{}:{}",
                source,
                event_type,
                timestamp.timestamp_millis(),
                sequence
            ),
            source: source.to_string(),
            event_type,
            level: kind.level(),
            summary: line_safe(summary),
            timestamp,
            timestamp_display: display_timestamp(timestamp),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, AppEventStoreState> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

impl AppEventStoreState {
    fn bootstrap(&self) -> AppEventBootstrap {
        AppEventBootstrap {
            snapshot: self.snapshot.clone(),
            recent_events: self.events.iter().cloned().collect(),
        }
    }

    fn snapshot_with_history(&self, mut snapshot: AppSnapshot) -> AppSnapshot {
        snapshot.notifications = self.notifications.iter().cloned().collect();
        snapshot.event_feed = self.events.iter().cloned().collect();
        snapshot
    }

    fn push_notification(&mut self, notification: NotificationView) {
        push_bounded(&mut self.notifications, self.capacity, notification);
        self.snapshot.notifications = self.notifications.iter().cloned().collect();
    }

    fn push_event(&mut self, item: EventFeedItem) {
        push_bounded(&mut self.events, self.capacity, item);
        self.snapshot.event_feed = self.events.iter().cloned().collect();
    }

    fn broadcast(&mut self, update: AppLiveUpdate) {
        self.subscribers
            .retain(|subscriber| subscriber.send(update.clone()).is_ok());
    }

    fn next_sequence(&mut self) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        sequence
    }
}

impl SystemEventKind {
    const fn source(&self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle",
            Self::Warning => "warning",
        }
    }

    const fn level(&self) -> u8 {
        match self {
            Self::Lifecycle => 1,
            Self::Warning => 2,
        }
    }
}

fn push_bounded<T>(items: &mut VecDeque<T>, capacity: usize, item: T) {
    if capacity == 0 {
        return;
    }
    if items.len() == capacity {
        items.pop_front();
    }
    items.push_back(item);
}
