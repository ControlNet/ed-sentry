use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::config::RuntimeConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatusKind {
    Disabled,
    Starting,
    Running,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatrixStartupStatus {
    pub kind: ServiceStatusKind,
    pub message: Option<String>,
    pub room_id: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebStartupStatus {
    pub kind: ServiceStatusKind,
    pub message: Option<String>,
    pub bind_address: Option<String>,
    pub url: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MatrixStatusView {
    pub kind: ServiceStatusKind,
    pub status_label: String,
    pub message: Option<String>,
    pub room_id: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub checked_at_display: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WebStatusView {
    pub kind: ServiceStatusKind,
    pub status_label: String,
    pub message: Option<String>,
    pub bind_address: Option<String>,
    pub url: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub checked_at_display: Option<String>,
}

impl MatrixStartupStatus {
    pub fn disabled() -> Self {
        Self {
            kind: ServiceStatusKind::Disabled,
            message: None,
            room_id: None,
            checked_at: None,
        }
    }

    pub fn running(room_id: impl Into<String>, checked_at: DateTime<Utc>) -> Self {
        Self {
            kind: ServiceStatusKind::Running,
            message: None,
            room_id: Some(room_id.into()),
            checked_at: Some(checked_at),
        }
    }

    pub fn warning(message: impl Into<String>, checked_at: DateTime<Utc>) -> Self {
        Self {
            kind: ServiceStatusKind::Warning,
            message: Some(message.into()),
            room_id: None,
            checked_at: Some(checked_at),
        }
    }

    pub fn from_runtime_config(config: &RuntimeConfig) -> Self {
        let matrix = config.matrix_runtime();
        if let Some(runtime) = matrix.config {
            return Self {
                kind: ServiceStatusKind::Starting,
                message: Some("Configured; connection pending".to_string()),
                room_id: Some(runtime.room_id),
                checked_at: None,
            };
        }
        if let Some(message) = matrix.warnings.into_iter().next() {
            return Self {
                kind: ServiceStatusKind::Warning,
                message: Some(message),
                room_id: None,
                checked_at: None,
            };
        }
        Self::disabled()
    }
}

impl WebStartupStatus {
    pub fn disabled() -> Self {
        Self {
            kind: ServiceStatusKind::Disabled,
            message: None,
            bind_address: None,
            url: None,
            checked_at: None,
        }
    }

    pub fn running(
        bind_address: impl Into<String>,
        url: impl Into<String>,
        checked_at: DateTime<Utc>,
    ) -> Self {
        Self {
            kind: ServiceStatusKind::Running,
            message: None,
            bind_address: Some(bind_address.into()),
            url: Some(url.into()),
            checked_at: Some(checked_at),
        }
    }

    pub fn warning(message: impl Into<String>, checked_at: DateTime<Utc>) -> Self {
        Self {
            kind: ServiceStatusKind::Warning,
            message: Some(message.into()),
            bind_address: None,
            url: None,
            checked_at: Some(checked_at),
        }
    }

    pub fn from_current_runtime_config(config: &RuntimeConfig) -> Self {
        if !config.web.enabled {
            return Self::disabled();
        }
        let bind_address = format!("{}:{}", config.web.host, config.web.port);
        Self {
            kind: ServiceStatusKind::Starting,
            message: Some("Configured; server startup pending".to_string()),
            bind_address: Some(bind_address),
            url: Some(format!("http://{}:{}", config.web.host, config.web.port)),
            checked_at: None,
        }
    }
}

impl From<MatrixStartupStatus> for MatrixStatusView {
    fn from(status: MatrixStartupStatus) -> Self {
        let checked_at_display = status.checked_at.map(display_timestamp);
        Self {
            kind: status.kind,
            status_label: status_label(status.kind),
            message: status.message,
            room_id: status.room_id,
            checked_at: status.checked_at,
            checked_at_display,
        }
    }
}

impl From<WebStartupStatus> for WebStatusView {
    fn from(status: WebStartupStatus) -> Self {
        let checked_at_display = status.checked_at.map(display_timestamp);
        Self {
            kind: status.kind,
            status_label: status_label(status.kind),
            message: status.message,
            bind_address: status.bind_address,
            url: status.url,
            checked_at: status.checked_at,
            checked_at_display,
        }
    }
}

fn status_label(kind: ServiceStatusKind) -> String {
    match kind {
        ServiceStatusKind::Disabled => "Disabled",
        ServiceStatusKind::Starting => "Starting",
        ServiceStatusKind::Running => "Running",
        ServiceStatusKind::Warning => "Warning",
        ServiceStatusKind::Error => "Error",
    }
    .to_string()
}

fn display_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
