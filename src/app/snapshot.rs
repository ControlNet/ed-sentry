use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::mission::MissionTracker;
use crate::state::SessionState;

use super::display::display_timestamp;
use super::{
    EventFeedItem, JournalSourceView, MatrixStartupStatus, MatrixStatusView, MissionListView,
    NotificationView, SessionView, WebStartupStatus, WebStatusView,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AppSnapshot {
    pub generated_at: DateTime<Utc>,
    pub generated_at_display: String,
    pub session: SessionView,
    pub missions: MissionListView,
    pub notifications: Vec<NotificationView>,
    pub event_feed: Vec<EventFeedItem>,
    pub journal_source: JournalSourceView,
    pub matrix: MatrixStatusView,
    pub web: WebStatusView,
}

impl AppSnapshot {
    pub fn from_state(
        state: &SessionState,
        missions: &MissionTracker,
        now: DateTime<Utc>,
        matrix: MatrixStartupStatus,
        web: WebStartupStatus,
    ) -> Self {
        Self {
            generated_at: now,
            generated_at_display: display_timestamp(now),
            session: SessionView::from_state(state, now),
            missions: MissionListView::from_tracker(missions),
            notifications: Vec::new(),
            event_feed: Vec::new(),
            journal_source: JournalSourceView::unknown(),
            matrix: matrix.into(),
            web: web.into(),
        }
    }
}
