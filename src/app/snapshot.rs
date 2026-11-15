use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::config::MonitorConfig;
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
    pub thresholds: WarningThresholdsView,
}

/// Monitor warning thresholds the dashboard draws against. They are copied from
/// the active monitor config so viewers without config access still see them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct WarningThresholdsView {
    pub warn_kill_rate_per_hour: u16,
    pub warn_no_kills_minutes: u16,
}

impl From<&MonitorConfig> for WarningThresholdsView {
    fn from(monitor: &MonitorConfig) -> Self {
        Self {
            warn_kill_rate_per_hour: monitor.warn_kill_rate,
            warn_no_kills_minutes: monitor.warn_no_kills_minutes,
        }
    }
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
            thresholds: WarningThresholdsView::from(&MonitorConfig::default()),
        }
    }

    pub fn with_thresholds(mut self, monitor: &MonitorConfig) -> Self {
        self.thresholds = WarningThresholdsView::from(monitor);
        self
    }

    pub fn with_tunnel_status(mut self, tunnel: TunnelStatus) -> Self {
        self.tunnel = tunnel.into();
        self
    }
}
