# Terminal Watch Loop Watcher Integration

- Terminal watch mode now starts the existing `AfkFileWatcherStart` after normal startup/replay separation in `src/app/runtime/terminal.rs`; replay remains independent.
- The terminal watcher loop lives in `src/app/runtime/terminal/watch_loop.rs` to keep `terminal.rs` under the 250 pure-LOC ceiling.
- `watch_loop::run` keeps the `AfkFileWatcher` guard alive and uses `tokio::select!` over watcher events plus `tokio::time::interval(runtime.poll_interval())` fallback ticks.
- `watch_runner::watcher_event_cycle` is the reusable helper for future desktop integration: selected Journal events call `poll_runtime_once`, companion events call `process_companion_update`, and watcher warnings produce sanitized warning batches.
- Deterministic tests live in `src/app/runtime/terminal/tests.rs` and cover immediate selected-file delivery plus checklist-only `Status.json` delivery without terminal output.
