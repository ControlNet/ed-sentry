# GUI/WebUI/Tauri Implementation Facts

Updated: 2026-06-20

## Architecture

- WebUI is config-gated by `[web] enabled`; defaults remain disabled and local-first.
- The CLI watch path constructs one `MonitorRuntime`, starts WebUI from that runtime's `AppEventStore`, then continues polling the same runtime. WebUI does not create a second Journal tail or monitor pipeline.
- Replay remains terminal-only. The replay path uses `scan_replay_startup`, terminal delivery, and replay notification processing without WebUI or Matrix startup.
- Tauri uses `ui/src-tauri/` and starts the shared Rust `DesktopRuntime`, which wraps `MonitorRuntime` and bridges `AppLiveUpdate` events into the frontend.

## WebUI Backend

- WebUI startup is best-effort. Disabled, missing assets, bind failures, and server-stop errors become warning status/messages rather than monitor-fatal errors.
- The backend event buffer is `AppEventStore`; new subscribers receive a bootstrap snapshot and recent events before live updates.
- WebSocket `/api/events` and HTTP snapshot/config endpoints use sanitized DTOs. Journal folder paths are redacted to display labels and raw Journal lines are not exposed.
- State-changing config writes are loopback/host/origin guarded for WebUI. Non-loopback binds can serve read status but config writes return forbidden.

## Config Editing And Privacy

- Config API exposes Matrix token presence as a boolean and accepts replacement/clear operations; it does not echo the raw token back.
- Rust `Debug` for Matrix config/runtime config redacts access tokens.
- Frontend/Tauri adapter error handling redacts bearer/token/password/authorization-shaped strings and home-directory paths before surfacing command errors.
- Smoke tests use synthetic fixture data only. Task-15 evidence scans check for real token/private-key patterns and raw Journal event lines.

## Frontend And Tauri

- The shared React/Vite frontend supports web and Tauri adapters.
- Tauri commands are `load_snapshot`, `load_config`, and `save_config`; desktop config defaults to the Tauri app config directory when no explicit config path is provided.
- `pnpm --dir ui tauri build` succeeded in this worktree and produced `ui/src-tauri/target/release/ed-sentry-gui`.

## Packaging

- Web assets are not embedded in the Rust binary.
- Lookup order is `ED_SENTRY_WEBUI_DIST`, executable sibling `webui/`, then repo-local `ui/dist`.
- Packaged asset smoke verifies a copied binary with sibling `webui/index.html` can serve `/`.

## Verification Commands

- Todo 15 gate-fix split facts: `scripts/smoke-webui.sh` is now a thin CLI dispatcher that sources `scripts/smoke-webui/common.sh`, `scripts/smoke-webui/api-scenarios.sh`, and `scripts/smoke-webui/dashboard-scenarios.sh`; `ui/e2e/tauri-string-errors.spec.ts` owns the grep-addressable Tauri string-error adapter cases; `src/app/runtime/service/snapshot.rs` owns private `MonitorRuntime` snapshot assembly.
- Todo 15 gate-fix LOC proof: `.omo/evidence/gui-webui-tauri/task-15-gate-fix-loc.txt` records all split target files under 250 pure LOC with no `SIZE_OK` exception.
- Todo 15 gate-fix full gate: `scripts/verify-gui-webui-tauri.sh > .omo/evidence/gui-webui-tauri/task-15-gate-fix-full-gate.txt` passed with `VERIFY_GUI_WEBUI_TAURI: pass`.
- `test -x scripts/verify-gui-webui-tauri.sh`
- `scripts/verify-gui-webui-tauri.sh > .omo/evidence/gui-webui-tauri/task-15-full-gate.txt`
- `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line > .omo/evidence/gui-webui-tauri/task-15-replay-isolation.txt`
- `scripts/smoke-webui.sh --scenario packaged-assets --evidence .omo/evidence/gui-webui-tauri/task-15-packaged-assets.http`
- `scripts/smoke-webui.sh --scenario production-dashboard --evidence .omo/evidence/gui-webui-tauri/task-15-production-dashboard.txt`
- Secret/privacy source and evidence scan recorded at `.omo/evidence/gui-webui-tauri/task-15-secret-scan.txt`.
- Cleanup/process check recorded at `.omo/evidence/gui-webui-tauri/task-15-cleanup-process-check.txt`.

## Known Limitations

- No GUI replay.
- No historical database or multi-day stored analytics.
- No auth/public remote mode.
- No chart library dashboard.
- No Matrix command handling or game automation.
