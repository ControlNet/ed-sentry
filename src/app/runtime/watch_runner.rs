use std::io::Write;

use chrono::{DateTime, Utc};

use crate::config::RuntimeConfig;
use crate::delivery::{DeliveryHub, DeliveryWarning};
use crate::terminal::set_platform_window_title;
use crate::text::line_safe;

use super::{
    deliver_notifications, deliver_terminal_notifications, publish_status,
    send_matrix_startup_header, MonitorRuntime, RuntimeBatch, RuntimeError, RuntimeStatusSnapshot,
};

#[derive(Clone, Copy)]
pub(super) enum TitleMode {
    Ignore,
    PlatformWindow,
}

pub(super) struct WatchCycle {
    batch: RuntimeBatch,
    status: RuntimeStatusSnapshot,
}

pub(super) async fn send_startup_header<W: Write>(
    delivery: &mut DeliveryHub<W>,
    config: &RuntimeConfig,
    runtime: &MonitorRuntime,
    program_started_at: DateTime<Utc>,
) -> Result<(), RuntimeError> {
    let warnings = send_matrix_startup_header(
        delivery,
        config,
        &runtime.startup().journal_file,
        program_started_at,
    )
    .await?;
    print_delivery_warnings(warnings);
    Ok(())
}

pub(super) async fn run_startup<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    config: &RuntimeConfig,
    program_started_at: DateTime<Utc>,
    title_mode: TitleMode,
) -> Result<(), RuntimeError> {
    let preload = runtime.process_preload(program_started_at);
    deliver_runtime_batch(delivery, &preload).await?;

    if config.reset_session {
        let reset = runtime.reset_session(Utc::now());
        deliver_runtime_batch(delivery, &reset).await?;
    }

    let start = runtime.start_monitor_if_preloaded(Utc::now());
    deliver_runtime_batch(delivery, &start).await?;

    let status = runtime.status_snapshot(Utc::now(), true);
    publish_runtime_status_snapshot(delivery, &status, title_mode).await
}

pub(super) fn poll_runtime_once(
    runtime: &mut MonitorRuntime,
    now: DateTime<Utc>,
) -> Result<WatchCycle, RuntimeError> {
    let batch = runtime.poll_once(now)?;
    let status = runtime.status_snapshot(now, false);
    Ok(WatchCycle { batch, status })
}

pub(super) async fn poll_and_deliver<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    title_mode: TitleMode,
) -> Result<(), RuntimeError> {
    let cycle = poll_runtime_once(runtime, Utc::now())?;
    deliver_watch_cycle(delivery, &cycle, title_mode).await
}

pub(super) async fn deliver_watch_cycle<W: Write>(
    delivery: &mut DeliveryHub<W>,
    cycle: &WatchCycle,
    title_mode: TitleMode,
) -> Result<(), RuntimeError> {
    deliver_runtime_batch(delivery, &cycle.batch).await?;
    publish_runtime_status_snapshot(delivery, &cycle.status, title_mode).await
}

async fn deliver_runtime_batch<W: Write>(
    delivery: &mut DeliveryHub<W>,
    batch: &RuntimeBatch,
) -> Result<(), RuntimeError> {
    for warning in &batch.warnings {
        eprintln!("Warning: {}", line_safe(warning));
    }
    for item in &batch.notifications {
        deliver_terminal_notifications(delivery, std::slice::from_ref(item))?;
        let warnings = deliver_notifications(delivery, std::slice::from_ref(item)).await?;
        print_delivery_warnings(warnings);
    }
    Ok(())
}

async fn publish_runtime_status_snapshot<W: Write>(
    delivery: &mut DeliveryHub<W>,
    status: &RuntimeStatusSnapshot,
    title_mode: TitleMode,
) -> Result<(), RuntimeError> {
    if matches!(title_mode, TitleMode::PlatformWindow) {
        if let Some(title) = status.dynamic_title.as_deref() {
            set_platform_window_title(title);
        }
    }
    let warnings = publish_status(delivery, status).await?;
    print_delivery_warnings(warnings);
    Ok(())
}

pub(super) fn print_delivery_warnings(warnings: Vec<DeliveryWarning>) {
    for warning in warnings {
        eprintln!("Warning: {}", line_safe(&warning.message));
    }
}
