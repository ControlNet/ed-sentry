use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};

use super::display::display_timestamp;

mod auth;
mod lifecycle;
mod provider;

pub use auth::{
    ActiveTunnel, TunnelAuth, TunnelAuthClaims, TunnelAuthError, TunnelAuthIssue,
    TunnelAuthPurpose, TunnelAuthToken, TunnelAuthValidation, TunnelAuthValidationResult,
    TunnelSigningSecret,
};
pub use lifecycle::TunnelLifecycleManager;
pub use provider::{
    cloudflare_trycloudflare_url, CloudflareQuickTunnelProvider, SshTunnelProvider, TunnelManager,
    TunnelProviderController,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelProvider {
    CloudflareQuick,
    Ssh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelStatusKind {
    Disabled,
    Start,
    Starting,
    Running,
    Error,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct TunnelSessionId(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TunnelSession {
    pub id: TunnelSessionId,
    pub public_url: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TunnelStatus {
    pub kind: TunnelStatusKind,
    pub provider: TunnelProvider,
    pub session_id: Option<TunnelSessionId>,
    pub message: Option<String>,
    pub public_url: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub retryable_error: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TunnelStatusView {
    pub kind: TunnelStatusKind,
    pub status_label: String,
    pub provider: TunnelProvider,
    pub provider_label: String,
    pub session_id: Option<TunnelSessionId>,
    pub message: Option<String>,
    pub public_url: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub checked_at_display: Option<String>,
    pub retryable_error: bool,
}

impl TunnelProvider {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CloudflareQuick => "Cloudflare Quick Tunnel",
            Self::Ssh => "SSH Tunnel",
        }
    }
}

impl TunnelSessionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TunnelSession {
    pub fn starting(id: TunnelSessionId, checked_at: DateTime<Utc>) -> Self {
        Self {
            id,
            public_url: None,
            checked_at,
        }
    }

    pub fn running(
        id: TunnelSessionId,
        public_url: impl Into<String>,
        checked_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            public_url: Some(public_url.into()),
            checked_at,
        }
    }
}

impl TunnelStatus {
    pub fn disabled(provider: TunnelProvider) -> Self {
        Self::new(TunnelStatusKind::Disabled, provider)
    }

    pub fn disabled_with_message(
        provider: TunnelProvider,
        message: impl Into<String>,
        checked_at: DateTime<Utc>,
    ) -> Self {
        Self {
            kind: TunnelStatusKind::Disabled,
            provider,
            session_id: None,
            message: Some(message.into()),
            public_url: None,
            checked_at: Some(checked_at),
            retryable_error: false,
        }
    }

    pub fn manual_start(provider: TunnelProvider) -> Self {
        Self::new(TunnelStatusKind::Start, provider)
    }

    pub fn starting(provider: TunnelProvider, session: TunnelSession) -> Self {
        Self::from_session(TunnelStatusKind::Starting, provider, session)
    }

    pub fn running(provider: TunnelProvider, session: TunnelSession) -> Self {
        Self::from_session(TunnelStatusKind::Running, provider, session)
    }

    pub fn error(
        provider: TunnelProvider,
        message: impl Into<String>,
        checked_at: DateTime<Utc>,
    ) -> Self {
        Self {
            kind: TunnelStatusKind::Error,
            provider,
            session_id: None,
            message: Some(message.into()),
            public_url: None,
            checked_at: Some(checked_at),
            retryable_error: false,
        }
    }

    pub fn retryable_error(
        provider: TunnelProvider,
        message: impl Into<String>,
        checked_at: DateTime<Utc>,
    ) -> Self {
        Self {
            retryable_error: true,
            ..Self::error(provider, message, checked_at)
        }
    }

    pub fn unsupported(provider: TunnelProvider, checked_at: DateTime<Utc>) -> Self {
        Self {
            kind: TunnelStatusKind::Unsupported,
            provider,
            session_id: None,
            message: Some(format!(
                "{} is not supported in this release",
                provider.label()
            )),
            public_url: None,
            checked_at: Some(checked_at),
            retryable_error: false,
        }
    }

    fn new(kind: TunnelStatusKind, provider: TunnelProvider) -> Self {
        Self {
            kind,
            provider,
            session_id: None,
            message: None,
            public_url: None,
            checked_at: None,
            retryable_error: false,
        }
    }

    fn from_session(
        kind: TunnelStatusKind,
        provider: TunnelProvider,
        session: TunnelSession,
    ) -> Self {
        Self {
            kind,
            provider,
            session_id: Some(session.id),
            message: None,
            public_url: session.public_url,
            checked_at: Some(session.checked_at),
            retryable_error: false,
        }
    }
}

impl Default for TunnelStatus {
    fn default() -> Self {
        Self::disabled(TunnelProvider::CloudflareQuick)
    }
}

impl From<TunnelStatus> for TunnelStatusView {
    fn from(status: TunnelStatus) -> Self {
        let checked_at_display = status.checked_at.map(display_timestamp);
        Self {
            kind: status.kind,
            status_label: tunnel_status_label(status.kind).to_string(),
            provider: status.provider,
            provider_label: status.provider.label().to_string(),
            session_id: status.session_id,
            message: status.message,
            public_url: status.public_url,
            checked_at: status.checked_at,
            checked_at_display,
            retryable_error: status.retryable_error,
        }
    }
}

impl Serialize for TunnelStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TunnelStatusView::from(self.clone()).serialize(serializer)
    }
}

const fn tunnel_status_label(kind: TunnelStatusKind) -> &'static str {
    match kind {
        TunnelStatusKind::Disabled => "Disabled",
        TunnelStatusKind::Start => "Start",
        TunnelStatusKind::Starting => "Starting",
        TunnelStatusKind::Running => "Running",
        TunnelStatusKind::Error => "Error",
        TunnelStatusKind::Unsupported => "Unsupported",
    }
}
