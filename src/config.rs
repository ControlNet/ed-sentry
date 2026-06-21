mod error;
mod journal;
mod log_levels;
mod matrix;
mod model;
mod monitor;
mod read;
mod runtime;
mod source;
mod value_read;
mod web;
mod write;

pub use error::ConfigError;
pub use journal::JournalConfig;
pub use log_levels::LogLevelConfig;
pub use matrix::{
    matrix_runtime_config, MatrixConfig, MatrixRuntimeConfig, MatrixRuntimeConfigResult,
    MATRIX_DEVICE_ID,
};
pub use model::{AppConfig, CliConfigOverrides, LoadedConfig, RuntimeConfig};
pub use monitor::MonitorConfig;
pub use source::{ConfigBlockReason, ConfigPath, ConfigSource, ConfigWriteState};
pub use web::WebConfig;
pub use write::{ConfigWriteError, ConfigWriteOutcome};

use value_read::{read_bool, read_optional_string, read_string, read_u16};

#[cfg(test)]
mod matrix_tests;
#[cfg(test)]
mod read_tests;
