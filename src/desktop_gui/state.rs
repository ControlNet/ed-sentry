use std::sync::Arc;

use tokio::sync::{watch, RwLock};

use crate::app::runtime::DesktopRuntime;
use crate::config::{ConfigSource, RuntimeConfig};

pub(crate) struct DesktopState {
    pub(crate) config: RwLock<RuntimeConfig>,
    pub(crate) config_source: RwLock<ConfigSource>,
    pub(crate) runtime: RwLock<Option<Arc<DesktopRuntime>>>,
    pub(crate) startup_error: RwLock<Option<String>>,
    pub(crate) startup_signal: watch::Sender<()>,
}

impl DesktopState {
    pub(crate) fn new(
        config: RuntimeConfig,
        config_source: ConfigSource,
        startup_error: Option<String>,
    ) -> Self {
        let (startup_signal, _receiver) = watch::channel(());
        Self {
            config: RwLock::new(config),
            config_source: RwLock::new(config_source),
            runtime: RwLock::new(None),
            startup_error: RwLock::new(startup_error),
            startup_signal,
        }
    }

    pub(crate) fn notify_startup_changed(&self) {
        self.startup_signal.send_replace(());
    }
}
