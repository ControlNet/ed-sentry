use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio::time::Interval;

use crate::app::{AppEventStore, AppSnapshot, MatrixStartupStatus, WebStartupStatus};
use crate::config::RuntimeConfig;
use crate::delivery::DeliveryHub;
use crate::terminal::TerminalNotifier;
use crate::text::line_safe;
use crate::time::TimeDisplayZone;
use crate::web::WebServer;

use super::file_watcher::{AfkFileWatcher, AfkFileWatcherStart, AfkWatcherEvent};
use super::watch_runner::{self, TitleMode, WatcherEventBuffer};
use super::{
    build_watch_delivery_with_terminal, start_webui_silent, ConfiguredJournalSelector,
    MonitorRuntime, RuntimeError,
};

#[cfg(test)]
mod tests;

struct DesktopFileWatcher {
    _watcher: Option<AfkFileWatcher>,
    events: Option<mpsc::Receiver<AfkWatcherEvent>>,
}

enum DesktopLoopWake {
    Events(Vec<AfkWatcherEvent>),
    WatcherClosed,
    Interval,
}

enum DesktopRawWake {
    Watcher(Option<AfkWatcherEvent>),
    Debounce,
    Interval,
}

pub struct DesktopRuntime {
    runtime: Arc<Mutex<MonitorRuntime>>,
    event_store: AppEventStore,
    _web_server: WebServer,
    monitor_task: JoinHandle<()>,
}

impl DesktopRuntime {
    pub async fn start(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let program_started_at = Utc::now();
        let mut selector = ConfiguredJournalSelector;
        let mut runtime = MonitorRuntime::start(
            &config,
            &mut selector,
            MatrixStartupStatus::from_runtime_config(&config),
            WebStartupStatus::from_current_runtime_config(&config),
        )?;
        let web_server = start_webui_silent(&config, &mut runtime).await;
        let delivery = desktop_delivery(&config).await;
        let super::WatchDelivery {
            hub: mut delivery,
            matrix_status,
        } = delivery;
        runtime.set_matrix_status(matrix_status);

        watch_runner::send_startup_header(&mut delivery, &config, &runtime, program_started_at)
            .await?;
        watch_runner::run_startup(
            &mut runtime,
            &mut delivery,
            &config,
            program_started_at,
            TitleMode::Ignore,
        )
        .await?;

        let event_store = runtime.event_store();
        let file_watcher = DesktopFileWatcher::start(&runtime.startup().journal_file);
        let runtime = Arc::new(Mutex::new(runtime));
        let delivery = Arc::new(Mutex::new(delivery));
        let monitor_task = tokio::spawn(run_monitor_loop(
            Arc::clone(&runtime),
            Arc::clone(&delivery),
            file_watcher,
        ));

        Ok(Self {
            runtime,
            event_store,
            _web_server: web_server,
            monitor_task,
        })
    }

    pub fn event_store(&self) -> AppEventStore {
        self.event_store.clone()
    }

    pub async fn snapshot(&self) -> AppSnapshot {
        self.runtime.lock().await.snapshot(Utc::now())
    }
}

impl DesktopFileWatcher {
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

impl Drop for DesktopRuntime {
    fn drop(&mut self) {
        self.monitor_task.abort();
    }
}

async fn desktop_delivery(config: &RuntimeConfig) -> super::WatchDelivery<io::Sink> {
    let zone = if config.monitor.use_utc {
        TimeDisplayZone::Utc
    } else {
        TimeDisplayZone::Local
    };
    build_watch_delivery_with_terminal(config, TerminalNotifier::plain(io::sink(), zone)).await
}

async fn run_monitor_loop<W>(
    runtime: Arc<Mutex<MonitorRuntime>>,
    delivery: Arc<Mutex<DeliveryHub<W>>>,
    mut file_watcher: DesktopFileWatcher,
) where
    W: Write + Send + 'static,
{
    let mut interval = tokio::time::interval(runtime.lock().await.poll_interval());
    let mut event_buffer = WatcherEventBuffer::new();
    interval.tick().await;

    loop {
        let wake =
            next_desktop_loop_wake(&mut file_watcher, &mut interval, &mut event_buffer).await;
        let result = match wake {
            DesktopLoopWake::Events(events) => {
                let mut result = Ok(());
                for event in events {
                    result = deliver_desktop_event(&runtime, &delivery, event).await;
                    if result.is_err() {
                        break;
                    }
                }
                result
            }
            DesktopLoopWake::WatcherClosed => {
                file_watcher.events = None;
                Ok(())
            }
            DesktopLoopWake::Interval => deliver_desktop_poll(&runtime, &delivery).await,
        };

        if let Err(error) = result {
            eprintln!(
                "Warning: Desktop runtime poll failed: {}",
                line_safe(&error.to_string())
            );
        }
    }
}

async fn next_desktop_loop_wake(
    file_watcher: &mut DesktopFileWatcher,
    interval: &mut Interval,
    event_buffer: &mut WatcherEventBuffer,
) -> DesktopLoopWake {
    let delay = event_buffer.next_delay(Instant::now());
    let raw_wake = match (file_watcher.events.as_mut(), delay) {
        (Some(events), Some(delay)) => {
            tokio::select! {
                event = events.recv() => DesktopRawWake::Watcher(event),
                _ = tokio::time::sleep(delay) => DesktopRawWake::Debounce,
                _ = interval.tick() => DesktopRawWake::Interval,
            }
        }
        (Some(events), None) => {
            tokio::select! {
                event = events.recv() => DesktopRawWake::Watcher(event),
                _ = interval.tick() => DesktopRawWake::Interval,
            }
        }
        (None, Some(delay)) => {
            tokio::select! {
                _ = tokio::time::sleep(delay) => DesktopRawWake::Debounce,
                _ = interval.tick() => DesktopRawWake::Interval,
            }
        }
        (None, None) => {
            interval.tick().await;
            DesktopRawWake::Interval
        }
    };
    match raw_wake {
        DesktopRawWake::Watcher(Some(event)) => {
            DesktopLoopWake::Events(event_buffer.accept(event, Instant::now()))
        }
        DesktopRawWake::Watcher(None) => DesktopLoopWake::WatcherClosed,
        DesktopRawWake::Debounce => {
            DesktopLoopWake::Events(event_buffer.drain_ready(Instant::now()))
        }
        DesktopRawWake::Interval => DesktopLoopWake::Interval,
    }
}

async fn deliver_desktop_poll<W>(
    runtime: &Arc<Mutex<MonitorRuntime>>,
    delivery: &Arc<Mutex<DeliveryHub<W>>>,
) -> Result<(), RuntimeError>
where
    W: Write,
{
    let now = Utc::now();
    let cycle = {
        let mut runtime = runtime.lock().await;
        watch_runner::poll_runtime_once(&mut runtime, now)?
    };
    let mut delivery = delivery.lock().await;
    watch_runner::deliver_watch_cycle(&mut delivery, &cycle, TitleMode::Ignore).await
}

async fn deliver_desktop_event<W>(
    runtime: &Arc<Mutex<MonitorRuntime>>,
    delivery: &Arc<Mutex<DeliveryHub<W>>>,
    event: AfkWatcherEvent,
) -> Result<(), RuntimeError>
where
    W: Write,
{
    watch_runner::settle_watcher_event(&event).await;
    let now = Utc::now();
    let cycle = {
        let mut runtime = runtime.lock().await;
        watch_runner::watcher_event_cycle(&mut runtime, event, now)?
    };
    let mut delivery = delivery.lock().await;
    watch_runner::deliver_watch_cycle(&mut delivery, &cycle, TitleMode::Ignore).await
}
