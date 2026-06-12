# Task 12 Warning Scheduler

- Warning scheduling lives in `src/monitor.rs` inside `EventMonitor`; `SessionState` remains the source of session activity, session start, kill count, last kill timestamp, and total kill rate.
- Public deterministic evaluation API: `EventMonitor::check_warnings_at(now: DateTime<Utc>, preload: bool) -> anyhow::Result<()>`.
- `preload = true`, inactive sessions, and missing session starts return without dispatching warnings.
- Initial no-kill warning uses `monitor.warn_no_kills_initial_minutes`, fires once when there are zero observed kills since session start, and records no-kill warning time.
- Later no-kill warning requires at least one kill, uses `monitor.warn_no_kills_minutes` since `state.last_kill_at`, and respects `monitor.warn_cooldown_minutes` since the last no-kill warning.
- Low-kill-rate warning requires at least one kill, at least the initial no-kill window elapsed, `state.total_kill_rate_per_hour_at(now) < monitor.warn_kill_rate`, and cooldown since the last low-rate warning.
- `EventMonitor::process_event` resets warning scheduler state when `SessionState.session_started_at` changes and clears low-rate cooldown eligibility when a new kill increments `SessionState.kills`.
- Warning notifications use event types `no_kills` and `kill_rate`, log levels `log_levels.no_kills` and `log_levels.kill_rate`, and dispatch through `NotificationDispatcher`, so level `0` suppresses delivery without breaking monitor state.
- Deterministic coverage is in `tests/warnings.rs`; required evidence commands are `cargo test warnings_no_kill_threshold --all > .omo/evidence/task-12-no-kill.txt` and `cargo test warnings_disabled_during_preload --all > .omo/evidence/task-12-preload.txt`.
