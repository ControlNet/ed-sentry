use super::{AppConfig, CliConfigOverrides, ConfigSource, RuntimeConfig};

impl AppConfig {
    pub fn into_runtime(self, overrides: &CliConfigOverrides) -> RuntimeConfig {
        self.into_runtime_with_source(ConfigSource::InMemory, overrides)
    }

    pub fn into_runtime_with_source(
        mut self,
        source: ConfigSource,
        overrides: &CliConfigOverrides,
    ) -> RuntimeConfig {
        if let Some(folder) = &overrides.journal_folder {
            self.journal.folder = folder.to_string_lossy().into_owned();
        }
        if let Some(poll_interval_ms) = overrides.poll_interval_ms {
            self.monitor.poll_interval_ms = poll_interval_ms;
        }
        if overrides.no_status_line {
            self.monitor.live_status = false;
        }

        RuntimeConfig {
            journal: self.journal,
            monitor: self.monitor,
            log_levels: self.log_levels,
            matrix: self.matrix,
            web: self.web,
            config_source: source,
            set_file: overrides.set_file.clone(),
            file_select: overrides.file_select,
            reset_session: overrides.reset_session,
            debug: overrides.debug,
        }
    }
}
