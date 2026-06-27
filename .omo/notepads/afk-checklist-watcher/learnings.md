
## 2026-06-26T11:04:39Z

- Added `notify = "8.2"` to `Cargo.toml` and let Cargo resolve the lockfile; `Cargo.lock` now contains `notify 8.2.0`.
- Happy-path metadata evidence is saved in `.omo/evidence/afk-checklist-watcher/task-1-cargo-metadata.txt`.
- Negative major-version-9 assertion fails as expected and is saved in `.omo/evidence/afk-checklist-watcher/task-1-negative-version-check.txt`.
## 2026-06-26 Task 2 parser foundation

- Added `src/app/afk_checklist.rs` as a parser/view foundation only; no `AppSnapshot`, runtime watcher, UI, or Journal `Status` event wiring was added in this task.
- The view model emits exactly three rows with ids `hardpoints_deployed`, `engine_pips_zero`, and `cargo_loaded`, using serialized states `pass`, `fail`, `unknown` and sources `Status.json`, `Cargo.json`, `unknown`.
- Malformed or missing companion JSON is intentionally collapsed to `unknown` rows so later runtime file IO can treat unreadable companion files as non-fatal state updates.

## 2026-06-26T11:49:38Z Task 3 snapshot contract

- Added top-level `AppSnapshot.afk_checklist` with the default `AfkChecklistState::unknown().to_view()` rows through the central `AppSnapshot::from_state` constructor.
- Exported AFK checklist view/state row types from `src/app.rs` for later runtime and UI schema work.
- The runtime service fixture now asserts serialized JSON includes top-level `afk_checklist` and preserves `session.ship_hull_display`; evidence is saved in `.omo/evidence/afk-checklist-watcher/task-3-runtime-snapshot.txt` and `.omo/evidence/afk-checklist-watcher/task-3-snapshot-contract.txt`.

## 2026-06-26T12:30:00Z Task 6 watcher abstraction

- Added `src/app/runtime/file_watcher.rs` as the non-integrated watcher seam; it owns `notify::RecommendedWatcher`, keeps the guard alive, and exposes a bounded `tokio::sync::mpsc` receiver for normalized app file events.
- The watcher watches only the selected file parent with `RecursiveMode::NonRecursive` and filters to the exact selected file path, `Status.json`, and `Cargo.json`; unrelated `Journal.*.log` files remain ignored, and explicit non-Journal `fixture.log` selections are accepted.
- Watcher initialization failure now returns a polling fallback surface with one sanitized warning string, so Todo 7/8 can continue the existing polling loop instead of crashing watch mode.
- Deterministic tests cover classification, explicit non-Journal selection, fallback warning sanitization, debounce coalescing, and retry of partial companion reads; evidence is saved in `.omo/evidence/afk-checklist-watcher/task-6-watcher-tests.txt` and `.omo/evidence/afk-checklist-watcher/task-6-filter-retry.txt`.

## 2026-06-26T12:05:37Z Task 4 startup companion files

- `MonitorRuntime::start` now derives the AFK companion directory from the selected Journal file parent and stores a startup `AfkChecklistState` on the runtime.
- Startup companion reads are intentionally non-fatal: absent, unreadable, or malformed `Status.json`/`Cargo.json` collapse through the parser into `unknown` rows instead of failing runtime startup.
- Runtime snapshots now override the default `AppSnapshot::from_state` unknown checklist with the runtime's stored startup checklist state; evidence is saved in `.omo/evidence/afk-checklist-watcher/task-4-startup-existing-files.txt` and `.omo/evidence/afk-checklist-watcher/task-4-missing-files.txt`.

## 2026-06-26T13:05:37Z Task 5 runtime companion updates

- Added `MonitorRuntime::process_companion_update(path, now)` for watcher-facing Status/Cargo updates; it rereads only the selected Journal parent companion path that matches the supplied path and publishes a snapshot only when `afk_checklist` changes.
- Added a Cargo journal-event fallback in runtime event processing; after normal `JournalEvent::Cargo` handling, the runtime rereads `Cargo.json` once so missed companion watcher events are recovered without terminal/Matrix notifications.
- Malformed or missing `Cargo.json` remains a non-fatal checklist state update: `cargo_loaded` becomes `unknown`, notifications stay unchanged, and evidence is saved in `.omo/evidence/afk-checklist-watcher/task-5-cargo-malformed-fallback.txt`.

## 2026-06-26T12:58:21Z Task 9 UI schema normalization

- Added strict TypeScript Zod parsing for top-level `appSnapshotSchema.afk_checklist`, including row `state` values `pass`, `fail`, `unknown` and source values `Status.json`, `Cargo.json`, `unknown`.
- Updated mock dashboard data with deterministic rows for `hardpoints_deployed`, `engine_pips_zero`, and `cargo_loaded` using the Rust parser details from the AFK checklist foundation.
- Captured the red phase for checklist-only updates: `pnpm --dir ui test:e2e -- --grep "afk checklist-only"` failed with `Expected: true` and `Received: false` before `stableSnapshotKey()` included `afk_checklist`.
- Final UI schema/store evidence is saved in `.omo/evidence/afk-checklist-watcher/task-9-ui-schema-normalization.txt`; missing-checklist schema rejection evidence is saved in `.omo/evidence/afk-checklist-watcher/task-9-schema-rejects-missing-checklist.txt`.

## 2026-06-26T13:39:19Z Task 7 terminal watcher loop

- Integrated terminal watch mode with the existing `AfkFileWatcherStart` abstraction; watcher startup failure emits one sanitized warning and falls back to the existing polling behavior.
- Added a terminal watch loop submodule that keeps the watcher guard alive and selects over watcher events plus `tokio::time::interval(runtime.poll_interval())` for fallback polling.
- Selected Journal events route through `MonitorRuntime::poll_once`, preserving the existing `LiveTail::poll(parse_journal_line)` parser/drain path; `Status.json` and `Cargo.json` events route through `MonitorRuntime::process_companion_update` for checklist-only snapshots.
- Added deterministic terminal watcher tests for immediate selected-file delivery and checklist-only Status updates without waiting for the interval.

## 2026-06-26T14:25:12Z Task 8 desktop watcher loop

- Integrated `DesktopRuntime` with the same watcher event stream plus `tokio::time::interval` fallback used by terminal watch mode, starting the watcher from `runtime.startup().journal_file` after startup processing and before spawning the monitor task.
- Desktop selected-file watcher events now route through `watch_runner::watcher_event_cycle`, preserving `LiveTail::poll(parse_journal_line)`; companion watcher events reuse `MonitorRuntime::process_companion_update` and deliver checklist snapshots without terminal or Matrix notifications.
- Desktop delivery remains separated from runtime file IO: tests hold the delivery mutex during a Status.json event and prove the runtime mutex can be reacquired before delivery is released.
- Fallback coverage now confirms watcher init failure emits one sanitized polling warning and that the desktop monitor loop still observes appended Journal output through interval polling.

## 2026-06-26T14:56:18Z Task 11 docs

- README is the right user-facing surface because the implemented behavior changes watch latency and the dashboard snapshot users see during terminal, WebUI, and desktop watch-capable runs.
- Documented that selected Journal, `Status.json`, and `Cargo.json` watcher events drive low-latency updates, while `poll_interval_ms` remains fallback and housekeeping.
- Documented tri-state `Checklist` labels for `Hardpoints deployed`, `Engine pips zero`, and `Cargo loaded`, and clarified that cargo readiness means non-empty ship cargo from `Cargo.json`, not market value.
- Sanitized older knowledge notes that used personal Journal folder wording or raw companion-file examples, keeping the implementation facts without private local details.

## 2026-06-26T15:19:37Z Task 12 final gates

- Final gate evidence is saved in `.omo/evidence/afk-checklist-watcher/task-12-final-gates.txt`; all required commands exited `0`: `cargo fmt --check`, `cargo metadata --format-version 1 >/tmp/ed-sentry-cargo-metadata.json`, `cargo test --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `pnpm --dir ui lint`, `pnpm --dir ui typecheck`, `pnpm --dir ui build`, `pnpm --dir ui test:e2e -- --grep "Checklist|dashboard store|adapter|operational regions"`, and `./scripts/package-windows-gnu.sh`.
- Windows GNU package artifacts were regenerated and verified at `dist/ed-sentry-x86_64-pc-windows-gnu.zip`, `dist/ed-sentry/ed-sentry.exe`, `dist/ed-sentry/ed-sentry-core.exe`, `dist/ed-sentry/WebView2Loader.dll`, and `dist/ed-sentry/webui/index.html`.
- Package output includes SHA-256 lines for the zip, `ed-sentry.exe`, `ed-sentry-core.exe`, `webui/index.html`, and `WebView2Loader.dll`; the artifact/hash verification block in the evidence file reports `ARTIFACT_AND_HASH_VERIFICATION: PASS`.
- Secret-guard staged and `.gitignore` coverage checks both exited `0`; no staged files were present and all common sensitive patterns are covered.

## 2026-06-26T17:16:31Z Final Wave F3 UNKNOWN UI evidence

- Added a deterministic WebUI mock scenario, `mock_state=afk_checklist_unknown`, so Playwright can prove the existing Checklist component visibly renders uppercase `UNKNOWN` for `Hardpoints deployed` while preserving `PASS` for `Engine pips zero` and `FAIL` for `Cargo loaded`.
- Kept the original `@todo10-dashboard renders all first-milestone operational regions` smoke test unchanged; the focused UNKNOWN test separately asserts the `Checklist` region is visible and `Ship Integrity` remains absent.
- Fresh evidence is saved in `.omo/evidence/afk-checklist-watcher/task-10-checklist-unknown-e2e.txt`, `.omo/evidence/afk-checklist-watcher/task-10-checklist-unknown-panel.png`, and `.omo/evidence/afk-checklist-watcher/fix-final-wave-ui-unknown.txt`; final package hash evidence still needs a later refresh after the backend debounce/retry fix lands.

## 2026-06-26T17:47:44Z Final Wave F1/F2 debounce retry blocker

- Final Wave F1/F2 was valid: `DebouncedWatcherEvents` and `CompanionReadRetry` were exported and unit-tested, but terminal and desktop watcher loops still forwarded companion events directly into `watcher_event_cycle`, and companion state was read once before publishing `unknown`.
- The fix moves helper usage into the real runtime seam: terminal and desktop loops now share `watch_runner::WatcherEventBuffer` for `Status.json` / `Cargo.json` coalescing, and both call `watch_runner::settle_watcher_event` before `watcher_event_cycle` so partial companion JSON gets delayed retry before the final runtime update.
- Desktop retry sleeps happen before acquiring `MonitorRuntime`, preserving the existing lock discipline; selected Journal events still go through `poll_runtime_once` / `LiveTail::poll(parse_journal_line)`, and watcher init fallback still emits one sanitized warning before interval polling.
- Evidence is saved in `.omo/evidence/afk-checklist-watcher/fix-final-wave-debounce-retry.txt`.

## 2026-06-26T18:34:25Z Task 12 final gates refresh

- Refreshed `.omo/evidence/afk-checklist-watcher/task-12-final-gates.txt` after the backend debounce/retry and UI UNKNOWN fixes; all nine required gates reran from repository root and exited `0`.
- Rebuilt the Windows GNU package with `./scripts/package-windows-gnu.sh`; verified required artifacts exist and recorded fresh SHA-256 hashes for the zip, `ed-sentry.exe`, `ed-sentry-core.exe`, `webui/index.html`, and `WebView2Loader.dll`.
- Secret-guard tracked and gitignore scans both exited `0`; final evidence reports `ARTIFACT_AND_HASH_VERIFICATION: PASS`, `SECRET_GUARD_VERIFICATION: PASS`, and `VERDICT: APPROVE`.
