use std::io;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::app::{AppEventStore, AppSnapshot, MatrixStartupStatus, WebStartupStatus};
use crate::config::RuntimeConfig;
use crate::delivery::DeliveryHub;
use crate::terminal::TerminalNotifier;
use crate::text::line_safe;
use crate::time::TimeDisplayZone;
use crate::web::WebServer;

use super::watch_runner::{self, TitleMode};
use super::{
    build_watch_delivery_with_terminal, start_webui_silent, ConfiguredJournalSelector,
    MonitorRuntime, RuntimeError,
};

type DesktopDelivery = DeliveryHub<io::Sink>;

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
        let runtime = Arc::new(Mutex::new(runtime));
        let delivery = Arc::new(Mutex::new(delivery));
        let monitor_task = tokio::spawn(run_monitor_loop(
            Arc::clone(&runtime),
            Arc::clone(&delivery),
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

async fn run_monitor_loop(
    runtime: Arc<Mutex<MonitorRuntime>>,
    delivery: Arc<Mutex<DesktopDelivery>>,
) {
    loop {
        let interval = runtime.lock().await.poll_interval();
        tokio::time::sleep(interval).await;
        if let Err(error) = deliver_desktop_cycle(&runtime, &delivery).await {
            eprintln!(
                "Warning: Desktop runtime poll failed: {}",
                line_safe(&error.to_string())
            );
        }
    }
}

async fn deliver_desktop_cycle(
    runtime: &Arc<Mutex<MonitorRuntime>>,
    delivery: &Arc<Mutex<DesktopDelivery>>,
) -> Result<(), RuntimeError> {
    let now = Utc::now();
    let cycle = {
        let mut runtime = runtime.lock().await;
        watch_runner::poll_runtime_once(&mut runtime, now)?
    };
    let mut delivery = delivery.lock().await;
    watch_runner::deliver_watch_cycle(&mut delivery, &cycle, TitleMode::Ignore).await
}
