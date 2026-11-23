use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::Utc;
use tokio::sync::{mpsc, watch};
use tokio::time::Interval;

use crate::config::RuntimeConfig;
use crate::delivery::DeliveryHub;
use crate::text::line_safe;
use crate::web::tunnel_state::WebTunnelState;

use super::super::file_watcher::{AfkFileWatcher, AfkFileWatcherStart, AfkWatcherEvent};
use super::super::watch_runner::{self, TitleMode, WatcherEventBuffer, WindowTitleTracker};
use super::super::{MonitorRuntime, RuntimeError};

pub(super) struct TerminalFileWatcher {
    pub(super) _watcher: Option<AfkFileWatcher>,
    pub(super) events: Option<mpsc::Receiver<AfkWatcherEvent>>,
}

enum TerminalLoopWake {
    Events(Vec<AfkWatcherEvent>),
    WatcherClosed,
    Interval,
    ConfigUpdated(Box<RuntimeConfig>),
}

enum TerminalRawWake {
    Watcher(Option<AfkWatcherEvent>),
    Debounce,
    Interval,
    ConfigUpdated(Box<RuntimeConfig>),
}

impl TerminalFileWatcher {
    fn start(selected_file: &Path) -> Self {
        match AfkFileWatcherStart::start(selected_file) {
            AfkFileWatcherStart::Watching { watcher, events } => Self {
                _watcher: Some(watcher),
                events: Some(events),
            },
            AfkFileWatcherStart::PollingFallback { warning } => {
                eprintln!("Warning: {}", line_safe(warning.message()));
                Self {
                    _watcher: None,
                    events: None,
                }
            }
        }
    }
}

pub(super) async fn run_with_config_updates<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    selected_file: &Path,
    config_rx: Option<watch::Receiver<RuntimeConfig>>,
    tunnel: Option<WebTunnelState>,
) -> Result<(), RuntimeError> {
    let file_watcher = TerminalFileWatcher::start(selected_file);
    run_loop_with_config_updates_and_tunnel(runtime, delivery, file_watcher, config_rx, tunnel)
        .await
}

#[cfg(test)]
pub(super) async fn run_loop<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    file_watcher: TerminalFileWatcher,
) -> Result<(), RuntimeError> {
    run_loop_with_config_updates(runtime, delivery, file_watcher, None).await
}

#[cfg(test)]
pub(super) async fn run_loop_with_config_updates<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    file_watcher: TerminalFileWatcher,
    config_rx: Option<watch::Receiver<RuntimeConfig>>,
) -> Result<(), RuntimeError> {
    run_loop_with_config_updates_and_tunnel(runtime, delivery, file_watcher, config_rx, None).await
}

async fn run_loop_with_config_updates_and_tunnel<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    file_watcher: TerminalFileWatcher,
    config_rx: Option<watch::Receiver<RuntimeConfig>>,
    tunnel: Option<WebTunnelState>,
) -> Result<(), RuntimeError> {
    let mut title_tracker = WindowTitleTracker::new(
        TitleMode::PlatformWindow,
        runtime.monitor_config().dynamic_title,
    );
    run_loop_with_tracker(
        runtime,
        delivery,
        file_watcher,
        config_rx,
        tunnel,
        &mut title_tracker,
    )
    .await
}

#[cfg(test)]
pub(super) async fn run_loop_with_config_updates_and_titles<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    file_watcher: TerminalFileWatcher,
    config_rx: Option<watch::Receiver<RuntimeConfig>>,
    titles: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
) -> Result<(), RuntimeError> {
    let mut title_tracker =
        WindowTitleTracker::recording(runtime.monitor_config().dynamic_title, titles);
    run_loop_with_tracker(
        runtime,
        delivery,
        file_watcher,
        config_rx,
        None,
        &mut title_tracker,
    )
    .await
}

async fn run_loop_with_tracker<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    mut file_watcher: TerminalFileWatcher,
    mut config_rx: Option<watch::Receiver<RuntimeConfig>>,
    tunnel: Option<WebTunnelState>,
    title_tracker: &mut WindowTitleTracker,
) -> Result<(), RuntimeError> {
    let mut interval = tokio::time::interval(runtime.poll_interval());
    let mut event_buffer = WatcherEventBuffer::new();
    interval.tick().await;

    loop {
        let wake = next_loop_wake(
            &mut file_watcher,
            &mut interval,
            &mut event_buffer,
            &mut config_rx,
        )
        .await;

        match wake {
            TerminalLoopWake::Events(events) => {
                for event in events {
                    deliver_event_with_tracker(runtime, delivery, event, title_tracker).await?;
                }
            }
            TerminalLoopWake::WatcherClosed => {
                file_watcher.events = None;
            }
            TerminalLoopWake::Interval => {
                if let Some(tunnel) = &tunnel {
                    let status = tunnel.status(Utc::now()).await;
                    runtime.sync_tunnel_status(status);
                }
                watch_runner::poll_and_deliver_with_tracker(runtime, delivery, title_tracker)
                    .await?;
            }
            TerminalLoopWake::ConfigUpdated(config) => {
                let now = Utc::now();
                runtime.apply_runtime_config(&config, now);
                delivery.update_monitor_config(runtime.monitor_config());
                let status = runtime.status_snapshot(now, false);
                watch_runner::publish_runtime_status_snapshot_with_tracker(
                    delivery,
                    &status,
                    title_tracker,
                )
                .await?;
                interval = tokio::time::interval(runtime.poll_interval());
                interval.tick().await;
            }
        }
    }
}

async fn next_loop_wake(
    file_watcher: &mut TerminalFileWatcher,
    interval: &mut Interval,
    event_buffer: &mut WatcherEventBuffer,
    config_rx: &mut Option<watch::Receiver<RuntimeConfig>>,
) -> TerminalLoopWake {
    let delay = event_buffer.next_delay(Instant::now());
    let raw_wake = tokio::select! {
        updated = wait_for_config_update(config_rx) => match updated {
            Some(config) => TerminalRawWake::ConfigUpdated(Box::new(config)),
            None => TerminalRawWake::Interval,
        },
        wake = next_terminal_file_or_timer_wake(file_watcher, interval, delay) => wake,
    };
    match raw_wake {
        TerminalRawWake::Watcher(Some(event)) => {
            TerminalLoopWake::Events(event_buffer.accept(event, Instant::now()))
        }
        TerminalRawWake::Watcher(None) => TerminalLoopWake::WatcherClosed,
        TerminalRawWake::Debounce => {
            TerminalLoopWake::Events(event_buffer.drain_ready(Instant::now()))
        }
        TerminalRawWake::Interval => TerminalLoopWake::Interval,
        TerminalRawWake::ConfigUpdated(config) => TerminalLoopWake::ConfigUpdated(config),
    }
}

async fn wait_for_config_update(
    config_rx: &mut Option<watch::Receiver<RuntimeConfig>>,
) -> Option<RuntimeConfig> {
    let Some(rx) = config_rx.as_mut() else {
        return std::future::pending().await;
    };
    if rx.changed().await.is_err() {
        *config_rx = None;
        return std::future::pending().await;
    }
    Some(rx.borrow_and_update().clone())
}

async fn next_terminal_file_or_timer_wake(
    file_watcher: &mut TerminalFileWatcher,
    interval: &mut Interval,
    delay: Option<Duration>,
) -> TerminalRawWake {
    match (file_watcher.events.as_mut(), delay) {
        (Some(events), Some(delay)) => {
            tokio::select! {
                event = events.recv() => TerminalRawWake::Watcher(event),
                _ = tokio::time::sleep(delay) => TerminalRawWake::Debounce,
                _ = interval.tick() => TerminalRawWake::Interval,
            }
        }
        (Some(events), None) => {
            tokio::select! {
                event = events.recv() => TerminalRawWake::Watcher(event),
                _ = interval.tick() => TerminalRawWake::Interval,
            }
        }
        (None, Some(delay)) => {
            tokio::select! {
                _ = tokio::time::sleep(delay) => TerminalRawWake::Debounce,
                _ = interval.tick() => TerminalRawWake::Interval,
            }
        }
        (None, None) => {
            interval.tick().await;
            TerminalRawWake::Interval
        }
    }
}

#[cfg(test)]
pub(super) async fn deliver_event<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    event: AfkWatcherEvent,
) -> Result<(), RuntimeError> {
    let mut tracker = WindowTitleTracker::new(
        TitleMode::PlatformWindow,
        runtime.monitor_config().dynamic_title,
    );
    deliver_event_with_tracker(runtime, delivery, event, &mut tracker).await
}

async fn deliver_event_with_tracker<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    event: AfkWatcherEvent,
    title_tracker: &mut WindowTitleTracker,
) -> Result<(), RuntimeError> {
    watch_runner::settle_watcher_event(&event).await;
    let cycle = watch_runner::watcher_event_cycle(runtime, event, Utc::now())?;
    watch_runner::deliver_watch_cycle_with_tracker(delivery, &cycle, title_tracker).await
}
