use std::io::Write;
use std::path::Path;
use std::time::Instant;

use chrono::Utc;
use tokio::sync::mpsc;
use tokio::time::Interval;

use crate::delivery::DeliveryHub;
use crate::text::line_safe;

use super::super::file_watcher::{AfkFileWatcher, AfkFileWatcherStart, AfkWatcherEvent};
use super::super::watch_runner::{self, TitleMode, WatcherEventBuffer};
use super::super::{MonitorRuntime, RuntimeError};

pub(super) struct TerminalFileWatcher {
    pub(super) _watcher: Option<AfkFileWatcher>,
    pub(super) events: Option<mpsc::Receiver<AfkWatcherEvent>>,
}

enum TerminalLoopWake {
    Events(Vec<AfkWatcherEvent>),
    WatcherClosed,
    Interval,
}

enum TerminalRawWake {
    Watcher(Option<AfkWatcherEvent>),
    Debounce,
    Interval,
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

pub(super) async fn run<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    selected_file: &Path,
) -> Result<(), RuntimeError> {
    let file_watcher = TerminalFileWatcher::start(selected_file);
    run_loop(runtime, delivery, file_watcher).await
}

pub(super) async fn run_loop<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    mut file_watcher: TerminalFileWatcher,
) -> Result<(), RuntimeError> {
    let mut interval = tokio::time::interval(runtime.poll_interval());
    let mut event_buffer = WatcherEventBuffer::new();
    interval.tick().await;

    loop {
        let wake = next_loop_wake(&mut file_watcher, &mut interval, &mut event_buffer).await;

        match wake {
            TerminalLoopWake::Events(events) => {
                for event in events {
                    deliver_event(runtime, delivery, event).await?;
                }
            }
            TerminalLoopWake::WatcherClosed => {
                file_watcher.events = None;
            }
            TerminalLoopWake::Interval => {
                watch_runner::poll_and_deliver(runtime, delivery, TitleMode::PlatformWindow)
                    .await?;
            }
        }
    }
}

async fn next_loop_wake(
    file_watcher: &mut TerminalFileWatcher,
    interval: &mut Interval,
    event_buffer: &mut WatcherEventBuffer,
) -> TerminalLoopWake {
    let delay = event_buffer.next_delay(Instant::now());
    let raw_wake = match (file_watcher.events.as_mut(), delay) {
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
    }
}

pub(super) async fn deliver_event<W: Write>(
    runtime: &mut MonitorRuntime,
    delivery: &mut DeliveryHub<W>,
    event: AfkWatcherEvent,
) -> Result<(), RuntimeError> {
    watch_runner::settle_watcher_event(&event).await;
    let cycle = watch_runner::watcher_event_cycle(runtime, event, Utc::now())?;
    watch_runner::deliver_watch_cycle(delivery, &cycle, TitleMode::PlatformWindow).await
}
