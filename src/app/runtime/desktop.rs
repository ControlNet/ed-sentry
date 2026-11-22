use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinHandle;
use tokio::time::Interval;

use crate::app::{AppEventStore, AppSnapshot, MatrixStartupStatus, TunnelStatus, WebStartupStatus};
use crate::config::RuntimeConfig;
use crate::delivery::DeliveryHub;
use crate::terminal::TerminalNotifier;
use crate::text::line_safe;
use crate::time::TimeDisplayZone;
use crate::web::tunnel_state::WebTunnelState;
use crate::web::{RuntimeConfigApplier, WebServer};

use super::file_watcher::{AfkFileWatcher, AfkFileWatcherStart, AfkWatcherEvent};
use super::watch_runner::{self, TitleMode, WatcherEventBuffer};
use super::{
    build_watch_delivery_with_terminal, start_webui_silent_with_applier, ConfiguredJournalSelector,
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
    PollIntervalChanged(Duration),
}

enum DesktopRawWake {
    Watcher(Option<AfkWatcherEvent>),
    Debounce,
    Interval,
    PollIntervalChanged(Duration),
}

struct DesktopConfigApplier {
    runtime: OnceLock<Arc<Mutex<MonitorRuntime>>>,
    poll_interval_tx: watch::Sender<Duration>,
}

#[async_trait]
impl RuntimeConfigApplier for DesktopConfigApplier {
    async fn apply_runtime_config(&self, config: &RuntimeConfig) {
        let Some(runtime) = self.runtime.get() else {
            return;
        };
        let poll_interval = {
            let mut runtime = runtime.lock().await;
            runtime.apply_runtime_config(config, Utc::now());
            runtime.poll_interval()
        };
        let _ = self.poll_interval_tx.send(poll_interval);
    }
}

pub struct DesktopRuntime {
    runtime: Arc<Mutex<MonitorRuntime>>,
    event_store: AppEventStore,
    _web_server: WebServer,
    tunnel: Option<WebTunnelState>,
    poll_interval_tx: watch::Sender<Duration>,
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
        let (poll_interval_tx, poll_interval_rx) = watch::channel(runtime.poll_interval());
        let applier = Arc::new(DesktopConfigApplier {
            runtime: OnceLock::new(),
            poll_interval_tx: poll_interval_tx.clone(),
        });
        let web_server = start_webui_silent_with_applier(
            &config,
            &mut runtime,
            Some(Arc::clone(&applier) as Arc<dyn RuntimeConfigApplier>),
        )
        .await;
        let tunnel =
            super::web::start_tunnel_after_webui(&config, &mut runtime, &web_server, true).await;
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
        let _ = applier.runtime.set(Arc::clone(&runtime));
        let delivery = Arc::new(Mutex::new(delivery));
        let monitor_task = tokio::spawn(run_monitor_loop_with_interval(
            Arc::clone(&runtime),
            Arc::clone(&delivery),
            file_watcher,
            Some(poll_interval_rx),
        ));

        Ok(Self {
            runtime,
            event_store,
            _web_server: web_server,
            tunnel,
            poll_interval_tx,
            monitor_task,
        })
    }

    pub fn event_store(&self) -> AppEventStore {
        self.event_store.clone()
    }

    pub async fn snapshot(&self) -> AppSnapshot {
        self.runtime.lock().await.snapshot(Utc::now())
    }

    pub async fn apply_runtime_config(&self, config: &RuntimeConfig) {
        let poll_interval = {
            let mut runtime = self.runtime.lock().await;
            runtime.apply_runtime_config(config, Utc::now());
            runtime.poll_interval()
        };
        let _ = self.poll_interval_tx.send(poll_interval);
        if let Some(tunnel) = &self.tunnel {
            tunnel.configure_provider(config).await;
        }
        self._web_server
            .sync_external_config(config, &config.config_source)
            .await;
    }

    pub async fn tunnel_is_running(&self) -> bool {
        let Some(tunnel) = &self.tunnel else {
            return false;
        };
        tunnel.active_tunnel(Utc::now()).await.is_some()
    }

    pub async fn tunnel_status(&self) -> TunnelStatus {
        let Some(tunnel) = &self.tunnel else {
            return TunnelStatus::default();
        };
        tunnel.status(Utc::now()).await
    }

    pub async fn start_tunnel(&self) -> TunnelStatus {
        let Some(tunnel) = &self.tunnel else {
            return TunnelStatus::default();
        };
        let status = tunnel.start(Utc::now()).await;
        self.runtime.lock().await.set_tunnel_status(status.clone());
        status
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

#[cfg(test)]
async fn run_monitor_loop<W>(
    runtime: Arc<Mutex<MonitorRuntime>>,
    delivery: Arc<Mutex<DeliveryHub<W>>>,
    file_watcher: DesktopFileWatcher,
) where
    W: Write + Send + 'static,
{
    run_monitor_loop_with_interval(runtime, delivery, file_watcher, None).await;
}

async fn run_monitor_loop_with_interval<W>(
    runtime: Arc<Mutex<MonitorRuntime>>,
    delivery: Arc<Mutex<DeliveryHub<W>>>,
    mut file_watcher: DesktopFileWatcher,
    mut poll_interval_rx: Option<watch::Receiver<Duration>>,
) where
    W: Write + Send + 'static,
{
    let mut interval = tokio::time::interval(runtime.lock().await.poll_interval());
    let mut event_buffer = WatcherEventBuffer::new();
    interval.tick().await;

    loop {
        let wake = next_desktop_loop_wake(
            &mut file_watcher,
            &mut interval,
            &mut event_buffer,
            &mut poll_interval_rx,
        )
        .await;
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
            DesktopLoopWake::PollIntervalChanged(duration) => {
                interval = tokio::time::interval(duration);
                interval.tick().await;
                Ok(())
            }
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
    poll_interval_rx: &mut Option<watch::Receiver<Duration>>,
) -> DesktopLoopWake {
    let delay = event_buffer.next_delay(Instant::now());
    let raw_wake = tokio::select! {
        changed = wait_for_poll_interval_change(poll_interval_rx) => match changed {
            Some(duration) => DesktopRawWake::PollIntervalChanged(duration),
            None => DesktopRawWake::Interval,
        },
        wake = next_file_or_timer_wake(file_watcher, interval, delay) => wake,
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
        DesktopRawWake::PollIntervalChanged(duration) => {
            DesktopLoopWake::PollIntervalChanged(duration)
        }
    }
}

async fn wait_for_poll_interval_change(
    poll_interval_rx: &mut Option<watch::Receiver<Duration>>,
) -> Option<Duration> {
    let Some(rx) = poll_interval_rx.as_mut() else {
        return std::future::pending().await;
    };
    if rx.changed().await.is_err() {
        *poll_interval_rx = None;
        return std::future::pending().await;
    }
    Some(*rx.borrow_and_update())
}

async fn next_file_or_timer_wake(
    file_watcher: &mut DesktopFileWatcher,
    interval: &mut Interval,
    delay: Option<Duration>,
) -> DesktopRawWake {
    match (file_watcher.events.as_mut(), delay) {
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
