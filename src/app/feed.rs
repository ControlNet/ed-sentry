use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::notifier::{AlertLevel, Notification};
use crate::text::line_safe;

use super::display::display_timestamp;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NotificationView {
    pub event_type: String,
    pub level: u8,
    pub alert_level: String,
    pub emoji: Option<String>,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub timestamp_display: String,
    pub mention: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EventFeedItem {
    pub id: String,
    pub source: String,
    pub event_type: String,
    pub level: u8,
    pub summary: String,
    pub timestamp: DateTime<Utc>,
    pub timestamp_display: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct JournalSourceView {
    pub folder: String,
    pub selected_file: Option<String>,
    pub recent_files: u16,
    pub status_label: String,
}

impl From<&Notification> for NotificationView {
    fn from(notification: &Notification) -> Self {
        Self {
            event_type: line_safe(&notification.event_type),
            level: notification.level,
            alert_level: alert_level_label(notification.alert_level),
            emoji: notification.emoji.as_deref().map(line_safe),
            text: line_safe(&notification.remote_text),
            timestamp: notification.timestamp,
            timestamp_display: display_timestamp(notification.timestamp),
            mention: notification.mention,
        }
    }
}

impl From<&Notification> for EventFeedItem {
    fn from(notification: &Notification) -> Self {
        let event_type = line_safe(&notification.event_type);
        let timestamp_display = display_timestamp(notification.timestamp);
        Self {
            id: format!("notification:{event_type}:{}", notification.timestamp),
            source: "notification".to_string(),
            event_type,
            level: notification.level,
            summary: line_safe(&notification.remote_text),
            timestamp: notification.timestamp,
            timestamp_display,
        }
    }
}

impl JournalSourceView {
    pub fn from_runtime_config(config: &crate::config::RuntimeConfig) -> Self {
        Self {
            folder: config.journal.folder.clone(),
            selected_file: config
                .set_file
                .as_ref()
                .map(|path| selected_file_display(path)),
            recent_files: config.journal.recent_files,
            status_label: "Configured".to_string(),
        }
    }

    pub(crate) fn unknown() -> Self {
        Self {
            folder: String::new(),
            selected_file: None,
            recent_files: 0,
            status_label: "Unknown".to_string(),
        }
    }
}

pub(crate) fn selected_file_display(path: &Path) -> String {
    path.file_name()
        .map(|name| line_safe(name.to_string_lossy().as_ref()))
        .unwrap_or_else(|| "<selected Journal file>".to_string())
}

fn alert_level_label(level: AlertLevel) -> String {
    match level {
        AlertLevel::Info => "info",
        AlertLevel::Warn => "warn",
        AlertLevel::Critical => "critical",
    }
    .to_string()
}
