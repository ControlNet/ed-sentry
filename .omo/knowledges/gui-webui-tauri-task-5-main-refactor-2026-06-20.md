# gui-webui-tauri Todo 5 main refactor

Date: 2026-06-20

## Useful facts

- The remaining Todo 5 blocker was `src/main.rs` being oversized and omitted from the pure LOC evidence.
- Failing-first evidence is `.omo/evidence/gui-webui-tauri/task-5-main-loc-failing-first.txt`; it captured `src/main.rs pure LOC: 527` before the refactor.
- `src/main.rs` is now a thin Tokio entrypoint that delegates to `ed_sentry::app::cli::run_from_env().await`.
- CLI parsing and mode dispatch live in `src/app/cli.rs`; former CLI unit coverage lives in `src/app/cli/tests.rs`.
- Terminal watch/replay behavior lives in `src/app/runtime/terminal.rs`.
- `src/app/runtime/terminal.rs` is intentionally focused on terminal runtime wiring and is close to the limit at 249 pure LOC. Future additions should split replay, watch loop, or prompt rendering first.
- `MonitorRuntime` remains the watch service owner. `src/main.rs` has no `LiveTail::from_preload` or `EventMonitor::from_runtime_config` references.
- Replay remains terminal-only. The captured manual run at `.omo/evidence/gui-webui-tauri/task-5-replay-regression.txt` contains `Scan`, `Kill`, `Total Stats`, `Monitor started`, and `Monitor stopped`, with no Web/Tauri startup text.

## Verification artifacts

- `.omo/evidence/gui-webui-tauri/task-5-pure-loc.txt`
- `.omo/evidence/gui-webui-tauri/task-5-code-review.md`
- `.omo/evidence/gui-webui-tauri/task-5-manual-qa-matrix.md`
- `.omo/evidence/gui-webui-tauri/task-5-cargo-fmt-check.txt`
- `.omo/evidence/gui-webui-tauri/task-5-runtime-service.txt`
- `.omo/evidence/gui-webui-tauri/task-5-replay-test.txt`
- `.omo/evidence/gui-webui-tauri/task-5-cli-config-watch-tail.txt`
- `.omo/evidence/gui-webui-tauri/task-5-live-tail.txt`
- `.omo/evidence/gui-webui-tauri/task-5-cargo-test-all.txt`
- `.omo/evidence/gui-webui-tauri/task-5-clippy.txt`
- `.omo/evidence/gui-webui-tauri/task-5-main-forbidden-grep.txt`
- `.omo/evidence/gui-webui-tauri/task-5-privacy-grep.txt`
- `.omo/evidence/gui-webui-tauri/task-5-replay-regression.txt`
