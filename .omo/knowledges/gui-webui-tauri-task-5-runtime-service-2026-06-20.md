# GUI WebUI Tauri Task 5 Runtime Service

Date: 2026-06-20

Todo 5 extracted watch-mode orchestration from `src/main.rs` into the exported app runtime service under `src/app/runtime.rs` and focused submodules:

- `src/app/runtime.rs`: core `MonitorRuntime`, preload processing, live polling, warning ticks, status snapshots, mission tracking, app snapshots.
- `src/app/runtime/types.rs`: runtime batch/snapshot/notification types plus journal selector abstraction.
- `src/app/runtime/delivery.rs`: terminal/Matrix delivery adapter, Matrix startup header, status publishing, status cadence.
- `src/app/runtime/delivery_debug.rs`: debug-only fake Matrix delivery driven by `ED_AFK_DASHBOARD_FAKE_MATRIX_*`.
- `src/app/runtime/paths.rs`: journal display helpers, Matrix warning redaction helpers, interactive selection helper.

`MonitorRuntime::start` selects and preloads the Journal file, constructs one `LiveTail`, one `EventMonitor`, and one `MissionTracker`, and exposes sanitized `AppSnapshot` values with notification and event-feed views. `process_preload`, `poll_once`, `reset_session`, `start_monitor_if_preloaded`, and `status_snapshot` let CLI, future WebUI, and future Tauri adapters consume one shared monitor pipeline instead of duplicating the watch loop.

`src/main.rs` remains responsible for CLI parsing, banner/startup display, interactive file selection prompt, replay mode, and entrypoint wiring. Watch mode now delegates runtime work to `MonitorRuntime` and delivery helpers.

Characterization/verification:

- Baseline before edit passed: `cargo test --test replay`, `cargo test --test cli_config cli_config_watch_tails_until_stopped`, and `cargo test --test live_tail live_tail_temp_file_drives_monitor_notifier_pipeline_without_sleeping`.
- Failing-first runtime-service test was added as `tests/runtime_service.rs`; it initially failed because `ed_sentry::app::runtime` did not exist, then passed after implementation.
- Manual QA artifacts:
  - `.omo/evidence/gui-webui-tauri/task-5-runtime-live-tail.txt`
  - `.omo/evidence/gui-webui-tauri/task-5-replay-regression.txt`
- Full gate passed: `cargo fmt --check`, focused replay/watch/live-tail/runtime tests, `cargo test --all`, and `cargo clippy --all-targets --all-features -- -D warnings`.

Notes for later todos:

- The runtime already emits sanitized snapshots, notification views, and event-feed items, but it does not yet maintain the fixed-size backend event buffer planned in Todo 6.
- `RuntimeBatch` is the natural bridge for future WebSocket or Tauri subscribers: it contains warnings, runtime notifications, and the current app snapshot.
- Matrix remains best-effort: delivery warnings are returned for CLI printing and raw access tokens are redacted by the runtime delivery adapter.
