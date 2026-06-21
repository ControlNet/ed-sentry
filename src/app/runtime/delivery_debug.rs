use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::config::MatrixRuntimeConfig;
use crate::delivery::RemoteDelivery;
use crate::notifier::Notification;
use crate::text::line_safe;

pub async fn debug_matrix_delivery_from_env(
    config: &MatrixRuntimeConfig,
) -> anyhow::Result<Option<Box<dyn RemoteDelivery>>> {
    let Some(log_path) = std::env::var_os("ED_AFK_DASHBOARD_FAKE_MATRIX_LOG") else {
        return Ok(None);
    };

    if let Ok(message) = std::env::var("ED_AFK_DASHBOARD_FAKE_MATRIX_CONNECT_ERROR") {
        anyhow::bail!(message);
    }

    let delivery = DebugFileMatrixDelivery::new(PathBuf::from(log_path), config.clone());
    delivery.append_record(serde_json::json!({
        "kind": "connect",
        "homeserver": &config.homeserver,
        "room_id": &config.room_id,
        "mention_user_id": &config.mention_user_id,
    }))?;
    Ok(Some(Box::new(delivery)))
}

struct DebugFileMatrixDelivery {
    log_path: PathBuf,
    access_token: String,
    send_error: Option<String>,
    status_error: Option<String>,
    send_delay: Duration,
    status_delay: Duration,
}

impl DebugFileMatrixDelivery {
    fn new(log_path: PathBuf, config: MatrixRuntimeConfig) -> Self {
        Self {
            log_path,
            access_token: config.access_token,
            send_error: std::env::var("ED_AFK_DASHBOARD_FAKE_MATRIX_SEND_ERROR").ok(),
            status_error: std::env::var("ED_AFK_DASHBOARD_FAKE_MATRIX_STATUS_ERROR").ok(),
            send_delay: debug_delay_from_env("ED_AFK_DASHBOARD_FAKE_MATRIX_SEND_DELAY_MS"),
            status_delay: debug_delay_from_env("ED_AFK_DASHBOARD_FAKE_MATRIX_STATUS_DELAY_MS"),
        }
    }

    fn append_record(&self, record: serde_json::Value) -> anyhow::Result<()> {
        use std::io::Write as _;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;
        writeln!(file, "{}", serde_json::to_string(&record)?)?;
        Ok(())
    }

    fn failure(&self, message: &str) -> anyhow::Error {
        anyhow::anyhow!(line_safe(message).replace(&self.access_token, "<redacted>"))
    }
}

#[async_trait::async_trait]
impl RemoteDelivery for DebugFileMatrixDelivery {
    async fn send(&mut self, notification: &Notification) -> anyhow::Result<()> {
        if !self.send_delay.is_zero() {
            tokio::time::sleep(self.send_delay).await;
        }
        if let Some(message) = &self.send_error {
            return Err(self.failure(message));
        }

        self.append_record(serde_json::json!({
            "kind": "send",
            "event_type": notification.event_type,
            "level": notification.level,
            "mention": notification.mention,
            "remote_text": notification.remote_text,
        }))
    }

    async fn publish_status(
        &mut self,
        status: &str,
        _now: DateTime<Utc>,
        force: bool,
    ) -> anyhow::Result<()> {
        if !self.status_delay.is_zero() {
            tokio::time::sleep(self.status_delay).await;
        }
        if let Some(message) = &self.status_error {
            return Err(self.failure(message));
        }

        self.append_record(serde_json::json!({
            "kind": "status",
            "status": line_safe(status),
            "force": force,
        }))
    }
}

fn debug_delay_from_env(name: &str) -> Duration {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::ZERO)
}
