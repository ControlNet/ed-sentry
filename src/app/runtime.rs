mod delivery;
#[cfg(debug_assertions)]
mod delivery_debug;
mod desktop;
mod paths;
mod service;
mod terminal;
mod types;
mod watch_runner;
mod web;

pub use delivery::{
    build_watch_delivery, build_watch_delivery_with_terminal, deliver_notifications,
    deliver_terminal_notifications, matrix_startup_label, publish_status,
    send_matrix_startup_header, status_cadence_from_config, WatchDelivery,
};
pub use desktop::DesktopRuntime;
pub use paths::{
    journal_basename, matrix_validation_reason, redact_matrix_error_message,
    selected_journal_from_choices, startup_commander, watch_journal_folder_display,
};
pub use service::MonitorRuntime;
pub use terminal::{run_replay, run_watch};
pub use types::{
    ConfiguredJournalSelector, JournalSelector, MonitorStartup, RuntimeBatch, RuntimeError,
    RuntimeNotification, RuntimeNotificationDelivery, RuntimeStatusSnapshot,
};
pub use web::{start_webui, start_webui_silent};
