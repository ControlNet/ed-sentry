use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::mission::MissionTracker;
use crate::state::SessionState;

use super::display::display_timestamp;
use super::{
    AfkChecklistState, AfkChecklistView, EventFeedItem, JournalSourceView, MatrixStartupStatus,
    MatrixStatusView, MissionListView, NotificationView, SessionView, TunnelStatus,
    TunnelStatusView, WebStartupStatus, WebStatusView,
};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AppSnapshot {
    pub generated_at: DateTime<Utc>,
    pub generated_at_display: String,
    pub session: SessionView,
    pub afk_checklist: AfkChecklistView,
    pub missions: MissionListView,
    pub notifications: Vec<NotificationView>,
    pub event_feed: Vec<EventFeedItem>,
    pub journal_source: JournalSourceView,
    pub matrix: MatrixStatusView,
    pub web: WebStatusView,
    pub tunnel: TunnelStatusView,
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
            afk_checklist: AfkChecklistState::unknown().to_view(),
            missions: MissionListView::from_tracker(missions),
            notifications: Vec::new(),
            event_feed: Vec::new(),
            journal_source: JournalSourceView::unknown(),
            matrix: matrix.into(),
            web: web.into(),
            tunnel: TunnelStatus::default().into(),
        }
    }

    pub fn with_tunnel_status(mut self, tunnel: TunnelStatus) -> Self {
        self.tunnel = tunnel.into();
        self
    }
}
