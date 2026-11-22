use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::{watch, RwLock};

use crate::app::runtime::DesktopRuntime;
use crate::config::{ConfigSource, RuntimeConfig};

pub(crate) struct DesktopState {
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) config_load_failed: AtomicBool,
    pub(crate) starting: AtomicBool,
    pub(crate) config: RwLock<RuntimeConfig>,
    pub(crate) running_config: RwLock<Option<RuntimeConfig>>,
    pub(crate) config_source: RwLock<ConfigSource>,
    pub(crate) runtime: RwLock<Option<Arc<DesktopRuntime>>>,
    pub(crate) startup_error: RwLock<Option<String>>,
    pub(crate) startup_signal: watch::Sender<()>,
}

impl DesktopState {
    pub(crate) fn new(
        config_path: Option<PathBuf>,
        config: RuntimeConfig,
        config_source: ConfigSource,
        startup_error: Option<String>,
        starting: bool,
    ) -> Self {
        let (startup_signal, _receiver) = watch::channel(());
        let config_load_failed = startup_error.is_some();
        Self {
            config_path,
            config_load_failed: AtomicBool::new(config_load_failed),
            starting: AtomicBool::new(starting),
            config: RwLock::new(config),
            running_config: RwLock::new(None),
            config_source: RwLock::new(config_source),
            runtime: RwLock::new(None),
            startup_error: RwLock::new(startup_error),
            startup_signal,
        }
    }

    pub(crate) fn is_starting(&self) -> bool {
        self.starting.load(Ordering::Acquire)
    }

    pub(crate) fn notify_startup_changed(&self) {
        self.startup_signal.send_replace(());
    }
}
