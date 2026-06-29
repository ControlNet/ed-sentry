use chrono::{DateTime, Utc};

use crate::app::{feed::selected_file_display, AppSnapshot, JournalSourceView};
use crate::text::line_safe;

use super::super::paths::snapshot_journal_folder_display;
use super::MonitorRuntime;

pub(super) fn runtime_snapshot(runtime: &MonitorRuntime, now: DateTime<Utc>) -> AppSnapshot {
    let mut snapshot = AppSnapshot::from_state(
        runtime.monitor.state(),
        &runtime.missions,
        now,
        runtime.matrix_status.clone(),
        runtime.web_status.clone(),
    )
    .with_tunnel_status(runtime.tunnel_status.clone());
    snapshot.session.commander = snapshot.session.commander.map(|value| line_safe(&value));
    snapshot.session.ship = snapshot.session.ship.map(|value| line_safe(&value));
    snapshot.session.system = snapshot.session.system.map(|value| line_safe(&value));
    snapshot.session.mode = snapshot.session.mode.map(|value| line_safe(&value));
    snapshot.afk_checklist = runtime.afk_checklist.to_view();
    snapshot.journal_source = JournalSourceView {
        folder: snapshot_journal_folder_display(&runtime.config),
        selected_file: Some(selected_file_display(&runtime.startup.journal_file)),
        recent_files: runtime.config.journal.recent_files,
        status_label: "Tailing".to_string(),
    };
    runtime.events.snapshot_with_history(snapshot)
}
