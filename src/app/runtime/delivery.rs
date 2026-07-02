use std::io::Write;
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::build_info::APP_BUILD_VERSION;
use crate::config::{MatrixRuntimeConfig, RuntimeConfig};
use crate::delivery::{DeliveryHub, DeliveryWarning, RemoteDelivery, StatusCadence};
use crate::matrix::MatrixDelivery;
use crate::notifier::{AlertLevel, Notification};
use crate::terminal::TerminalNotifier;
use crate::text::line_safe;

use super::{
    journal_basename, matrix_validation_reason, redact_matrix_error_message,
    watch_journal_folder_display, RuntimeError, RuntimeNotification, RuntimeNotificationDelivery,
    RuntimeStatusSnapshot,
};
use crate::app::{MatrixStartupStatus, ServiceStatusKind};

#[cfg(debug_assertions)]
use super::delivery_debug::debug_matrix_delivery_from_env;

pub struct WatchDelivery<W: Write> {
    pub hub: DeliveryHub<W>,
    pub matrix_status: MatrixStartupStatus,
}

pub async fn build_watch_delivery(config: &RuntimeConfig) -> WatchDelivery<std::io::Stdout> {
    let terminal = TerminalNotifier::stdout(&config.monitor);
    build_watch_delivery_with_terminal(config, terminal).await
}

pub async fn build_watch_delivery_with_terminal<W: Write>(
    config: &RuntimeConfig,
    terminal: TerminalNotifier<W>,
) -> WatchDelivery<W> {
    let (matrix_status, matrix) = connect_matrix_delivery(config).await;
    WatchDelivery {
        hub: DeliveryHub::new(terminal, matrix)
            .with_status_cadence(status_cadence_from_config(config)),
        matrix_status,
    }
}

pub async fn send_matrix_startup_header<W: Write>(
    delivery: &mut DeliveryHub<W>,
    config: &RuntimeConfig,
    set_file: &Path,
    program_started_at: DateTime<Utc>,
) -> Result<Vec<DeliveryWarning>, RuntimeError> {
    let notifications = [matrix_startup_header_notification(
        config,
        set_file,
        program_started_at,
    )];
    delivery
        .send_remote_notifications(&notifications)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))
}

pub async fn deliver_notifications<W: Write>(
    delivery: &mut DeliveryHub<W>,
    notifications: &[RuntimeNotification],
) -> Result<Vec<DeliveryWarning>, RuntimeError> {
    let all = notifications
        .iter()
        .filter(|item| item.delivery == RuntimeNotificationDelivery::All)
        .map(|item| item.notification.clone())
        .collect::<Vec<_>>();
    delivery
        .send_notifications(&all)
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))
}

pub fn deliver_terminal_notifications<W: Write>(
    delivery: &mut DeliveryHub<W>,
    notifications: &[RuntimeNotification],
) -> Result<(), RuntimeError> {
    let terminal_only = notifications
        .iter()
        .filter(|item| item.delivery == RuntimeNotificationDelivery::TerminalOnly)
        .map(|item| item.notification.clone())
        .collect::<Vec<_>>();
    delivery
        .send_terminal_notifications(&terminal_only)
        .map_err(|error| RuntimeError::new(error.to_string()))
}

pub async fn publish_status<W: Write>(
    delivery: &mut DeliveryHub<W>,
    status: &RuntimeStatusSnapshot,
) -> Result<Vec<DeliveryWarning>, RuntimeError> {
    let Some(status_line) = status.status_line.as_deref() else {
        return Ok(Vec::new());
    };
    let render_terminal_status = delivery.supports_status_line();
    delivery
        .publish_status(
            status_line,
            status.snapshot.generated_at,
            status.force_publish,
            render_terminal_status,
        )
        .await
        .map_err(|error| RuntimeError::new(error.to_string()))
}

pub fn status_cadence_from_config(config: &RuntimeConfig) -> StatusCadence {
    StatusCadence::new(Duration::from_secs(
        config
            .matrix
            .as_ref()
            .map(|matrix| matrix.status_update_interval_seconds)
            .unwrap_or(60),
    ))
}

pub fn matrix_startup_label(status: &MatrixStartupStatus) -> &'static str {
    match status.kind {
        ServiceStatusKind::Disabled => "disabled",
        ServiceStatusKind::Running => "enabled",
        ServiceStatusKind::Starting | ServiceStatusKind::Warning | ServiceStatusKind::Error => {
            "unavailable"
        }
    }
}

async fn connect_matrix_delivery(
    config: &RuntimeConfig,
) -> (MatrixStartupStatus, Option<Box<dyn RemoteDelivery>>) {
    let runtime = config.matrix_runtime();
    for warning in runtime.warnings {
        eprintln!(
            "Warning: Matrix delivery disabled: {}",
            line_safe(&matrix_validation_reason(&warning))
        );
    }

    let Some(matrix_config) = runtime.config else {
        let status = if config.matrix.is_some() {
            MatrixStartupStatus::warning("Matrix config incomplete", Utc::now())
        } else {
            MatrixStartupStatus::disabled()
        };
        return (status, None);
    };

    match connect_matrix_delivery_runtime(matrix_config.clone()).await {
        Ok(matrix) => (
            MatrixStartupStatus::running(matrix_config.room_id, Utc::now()),
            Some(matrix),
        ),
        Err(error) => {
            eprintln!(
                "Warning: Matrix delivery disabled: {}",
                redact_matrix_error_message(&error, &matrix_config.access_token)
            );
            (
                MatrixStartupStatus::warning("Matrix connection unavailable", Utc::now()),
                None,
            )
        }
    }
}

async fn connect_matrix_delivery_runtime(
    config: MatrixRuntimeConfig,
) -> anyhow::Result<Box<dyn RemoteDelivery>> {
    #[cfg(debug_assertions)]
    if let Some(delivery) = debug_matrix_delivery_from_env(&config).await? {
        return Ok(delivery);
    }

    Ok(Box::new(MatrixDelivery::connect(config).await?))
}

fn matrix_startup_header_notification(
    config: &RuntimeConfig,
    set_file: &Path,
    program_started_at: DateTime<Utc>,
) -> Notification {
    let matrix_room = config
        .matrix
        .as_ref()
        .and_then(|matrix| matrix.room_id.as_deref())
        .unwrap_or("[disabled]");
    let remote_text = format!(
        "🛰️ ed-sentry started\nVersion: {}\nStarted at: {}\nJournal folder: {}\nJournal file: {}\nMatrix room: {}",
        APP_BUILD_VERSION,
        program_started_at.to_rfc3339(),
        watch_journal_folder_display(config),
        journal_basename(set_file),
        matrix_room,
    );

    Notification::new(
        "matrix_startup",
        1,
        AlertLevel::Info,
        None,
        remote_text.clone(),
        remote_text,
        program_started_at,
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::build_info::APP_BUILD_VERSION;
    use crate::config::{AppConfig, CliConfigOverrides};

    #[test]
    fn matrix_startup_header_uses_build_version() {
        let config = AppConfig::default().into_runtime(&CliConfigOverrides::default());
        let started_at = Utc.with_ymd_and_hms(2035, 6, 9, 16, 30, 0).unwrap();

        let notification = matrix_startup_header_notification(
            &config,
            Path::new("Journal.2035-06-09T163000.01.log"),
            started_at,
        );

        assert!(
            notification
                .remote_text
                .contains(&format!("Version: {APP_BUILD_VERSION}")),
            "{}",
            notification.remote_text
        );
    }
}
