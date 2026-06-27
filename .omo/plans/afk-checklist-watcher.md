# afk-checklist-watcher - Work Plan

## TL;DR (For humans)

**What you'll get:** The app will update a new AFK readiness checklist as soon as Elite Dangerous changes the active Journal, `Status.json`, or `Cargo.json`. The dashboard's old `Ship Integrity` card will become a `Checklist` card with hardpoints, engine pips, and cargo readiness rows.

**Why this approach:** File watching gives low-latency updates without lowering the global polling interval or rewriting the proven Journal tail parser. Existing polling remains as a safety net for missed filesystem events and normal warning/status housekeeping.

**What it will NOT do:** It will not estimate cargo market value, auto-switch to a newer Journal file, add new watcher config options, or perform any game automation/relog/key simulation.

**Effort:** Medium
**Risk:** Medium - the main risk is coordinating cross-platform file watcher events with the existing async terminal/desktop loops and strict Rust/TypeScript snapshot schemas.
**Decisions to sanity-check:** Use `notify = "8.2"`; expose `afk_checklist` as a top-level snapshot field; use tri-state `PASS`/`FAIL`/`UNKNOWN`; cargo readiness means ship cargo is non-empty, not value-threshold based.

Your next move: either start implementation with `$start-work`, or ask for a high-accuracy dual Momus review of this plan first. Full execution detail follows below.

---

> TL;DR (machine): Medium-risk watcher/runtime/schema/UI feature; add `notify` watcher, companion parsers, top-level `afk_checklist`, Checklist UI, automated Rust/UI/package verification.

## Scope

### Must have

- Add Rust parsing/state/view support for AFK checklist companion telemetry from `Status.json` and `Cargo.json`.
- Add a top-level `AppSnapshot.afk_checklist` field with exactly three rows in this iteration:
  - `hardpoints_deployed`
  - `engine_pips_zero`
  - `cargo_loaded`
- Use this exact row contract in Rust serialization and TypeScript Zod parsing:
  - `state: "pass" | "fail" | "unknown"`
  - `label: string`
  - `detail: string`
  - `source: "Status.json" | "Cargo.json" | "unknown"`
- Use these exact checklist state rules:
  - `hardpoints_deployed`
    - pass: `Status.json` has integer `Flags` and `(Flags & 0x40) != 0`
    - fail: `Status.json` has integer `Flags` and `(Flags & 0x40) == 0`
    - unknown: `Status.json` missing, unreadable, malformed, or `Flags` absent/non-integer
  - `engine_pips_zero`
    - pass: `Status.json` has valid `Pips` array and `Pips[1] == 0`
    - fail: `Status.json` has valid `Pips` array and `Pips[1] > 0`
    - unknown: `Status.json` missing, unreadable, malformed, `Pips` absent, or `Pips` length < 2
  - `cargo_loaded`
    - pass: `Cargo.json` has `Vessel == "Ship"` and (`Count > 0` or `Inventory` is non-empty)
    - fail: `Cargo.json` has `Vessel == "Ship"`, `Count == 0`, and empty/missing `Inventory`
    - unknown: `Cargo.json` missing, unreadable, malformed, or not for `Vessel == "Ship"`
- Add `notify = "8.2"` to `Cargo.toml` and accept the corresponding `Cargo.lock` resolver changes.
- Watch the selected Journal file's parent directory non-recursively and filter events for:
  - the exact selected Journal file path (or selected non-Journal `--set-file` path when applicable),
  - `Status.json`,
  - `Cargo.json`.
- Trigger existing `LiveTail::poll(parse_journal_line)` immediately for selected Journal-file changes; do not rewrite the append parser/drain logic.
- Keep `poll_interval_ms` as a fallback/housekeeping cadence for missed watcher events, status publication, and warning checks.
- Read existing `Status.json` and `Cargo.json` once during runtime startup so the first snapshot is useful before any watcher event fires.
- Add Cargo journal-event fallback: when `JournalEvent::Cargo` is processed, non-fatally reread `Cargo.json` once and update checklist state if it changed.
- Publish snapshots for checklist-only changes without producing terminal or Matrix notifications, except sanitized runtime warnings for unrecoverable companion-read errors.
- Update browser and Tauri snapshot schema/mocks/normalization so checklist-only live updates are not ignored.
- Replace the `TacticalTelemetryView` panel titled `Ship Integrity` with an accessible panel titled `Checklist` containing visible rows:
  - `Hardpoints deployed`
  - `Engine pips zero`
  - `Cargo loaded`
  - each row renders `PASS`, `FAIL`, or `UNKNOWN`.
- Update dashboard and adapter boundary tests to assert `Checklist` behavior and removal of the `Ship Integrity` region.
- Because frontend/Tauri/WebUI asset files change, rebuild the Windows GNU package before reporting completion.

### Must NOT have (guardrails, anti-slop, scope boundaries)

- Do not modify `reference-design/design1.tsx`.
- Do not implement cargo market-value estimation or cargo value thresholds.
- Do not add `Market.json`, `NavRoute.json`, `ModulesInfo.json`, or other companion file watchers.
- Do not implement Journal auto-rotation or auto-switching to a newer Journal file.
- Do not add new CLI flags or config keys for watcher behavior.
- Do not replace the existing `LiveTail` append parser/drain implementation.
- Do not break explicit `--set-file` paths that are not named `Journal.*.log`; companion files still resolve from the selected file parent directory.
- Do not remove existing `session.ship_hull_percent`, `session.shields_up`, `session.fighter_alive`, or any current session fields from the snapshot contract.
- Do not expose raw local Journal paths or private local file contents in UI/logs/tests/evidence.
- Do not add game automation, key simulation, auto-relog, or action-taking behavior.

## Verification strategy

> Zero human intervention - all verification is agent-executed.

- Test decision: TDD/tests-after hybrid. Add focused failing tests before each parser/runtime/UI behavior where practical; for wiring-only changes, add tests immediately with the implementation in the same todo.
- Evidence root: `.omo/evidence/afk-checklist-watcher/`
- Required command gates before completion:
  - `cargo fmt --check`
  - `cargo metadata --format-version 1 >/tmp/ed-sentry-cargo-metadata.json`
  - `cargo test --all`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `pnpm --dir ui lint`
  - `pnpm --dir ui typecheck`
  - `pnpm --dir ui build`
  - `pnpm --dir ui test:e2e -- --grep "Checklist|dashboard store|adapter|operational regions"`
  - `./scripts/package-windows-gnu.sh`
- Expected final package signals:
  - `dist/ed-sentry-x86_64-pc-windows-gnu.zip` exists.
  - `dist/ed-sentry/ed-sentry.exe` exists.
  - `dist/ed-sentry/ed-sentry-core.exe` exists.
  - `dist/ed-sentry/WebView2Loader.dll` exists.
  - `dist/ed-sentry/webui/index.html` exists.
  - package script prints `sha256sum` lines for the zip, `ed-sentry.exe`, `ed-sentry-core.exe`, `webui/index.html`, and `WebView2Loader.dll`.

## Execution strategy

### Parallel execution waves

- Wave 1 - schema/parsing foundation:
  - Todo 1 dependency update.
  - Todo 2 checklist parser/state/view module.
  - Todo 3 snapshot DTO/export integration.
- Wave 2 - runtime integration:
  - Todo 4 startup companion read.
  - Todo 5 runtime companion update methods and Cargo-event fallback.
  - Todo 6 watcher abstraction and unit tests.
  - Todo 7 terminal loop integration.
  - Todo 8 desktop loop integration.
- Wave 3 - frontend contract/UI:
  - Todo 9 TypeScript schema/mocks/normalization tests.
  - Todo 10 Checklist panel replacement and dashboard e2e.
- Wave 4 - docs/evidence/final gates:
  - Todo 11 README/knowledge updates if implementation changes user-visible behavior.
  - Todo 12 full automated verification and Windows package rebuild.

### Dependency matrix

| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1 | none | 6, 7, 8, 12 | 2, 3 |
| 2 | none | 3, 4, 5 | 1 |
| 3 | 2 | 4, 5, 9, 12 | 1 |
| 4 | 2, 3 | 7, 8, 12 | 6 |
| 5 | 2, 3 | 7, 8, 12 | 6 |
| 6 | 1 | 7, 8, 12 | 4, 5, 9 |
| 7 | 4, 5, 6 | 12 | 8, 10 |
| 8 | 4, 5, 6 | 12 | 7, 10 |
| 9 | 3 | 10, 12 | 6 |
| 10 | 9 | 12 | 7, 8 |
| 11 | 1-10 | 12 | none |
| 12 | 1-11 | final delivery | none |

## Todos

> Implementation + Test = ONE todo. Never separate.
<!-- APPEND TASK BATCHES BELOW THIS LINE WITH edit/apply_patch - never rewrite the headers above. -->

- [x] 1. Add stable watcher dependency and lockfile update
  What to do / Must NOT do: Add `notify = "8.2"` under `[dependencies]` in `Cargo.toml` and allow `Cargo.lock` to update through normal Cargo resolution. Do not use `notify = "9"` or a release-candidate version. Do not add watcher config keys or CLI flags.
  Parallelization: Wave 1 | Blocked by: none | Blocks: 6, 7, 8, 12
  References (executor has NO interview context - be exhaustive): `Cargo.toml:13-28` current dependency list; `.omo/drafts/afk-checklist-watcher.md:35,45` dependency decision; Metis finding that `Cargo.lock` must be included.
  Acceptance criteria (agent-executable): `cargo metadata --format-version 1 >/tmp/ed-sentry-cargo-metadata.json` exits 0 and `/tmp/ed-sentry-cargo-metadata.json` contains a `notify` package with major version 8; `Cargo.toml` contains exactly a stable `notify = "8.2"` dependency line.
  QA scenarios (name the exact tool + invocation): happy: `cargo metadata --format-version 1 >/tmp/ed-sentry-cargo-metadata.json && python - <<'PY'\nimport json\nmeta=json.load(open('/tmp/ed-sentry-cargo-metadata.json'))\nversions=[p['version'] for p in meta['packages'] if p['name']=='notify']\nassert versions and all(v.split('.')[0]=='8' for v in versions), versions\nPY` writes `.omo/evidence/afk-checklist-watcher/task-1-cargo-metadata.txt`; failure: temporarily querying for major version 9 in the same Python assertion fails, proving the check catches the wrong dependency, and record output in `.omo/evidence/afk-checklist-watcher/task-1-negative-version-check.txt`.
  Commit: Y | build(deps): add notify watcher dependency

- [x] 2. Add AFK checklist parser, state, and view model tests
  What to do / Must NOT do: Add a small Rust module such as `src/app/afk_checklist.rs` or `src/app/runtime/afk_checklist.rs` that owns companion-file deserialization, derived state, and serializable view rows. It must deserialize only the fields needed now: `Status.json` `Flags`/`Pips`; `Cargo.json` `Vessel`/`Count`/`Inventory`. It must parse without panics and convert missing/malformed inputs to `unknown`. Do not add a Journal `Status` event and do not reuse the line-oriented Journal parser for whole companion files.
  Parallelization: Wave 1 | Blocked by: none | Blocks: 3, 4, 5
  References (executor has NO interview context - be exhaustive): `src/event.rs:502-523` existing cargo inventory/event shape; `src/event.rs:1541-1555` Cargo journal event parser; `.omo/knowledges/journal-status-ship-state-capabilities-2026-06-26.md` Status/Cargo capabilities; `.omo/drafts/afk-checklist-watcher.md:14,23-28` parser and tri-state decisions; Scope checklist state rules in this plan.
  Acceptance criteria (agent-executable): New Rust tests cover: hardpoints bit `0x40`; hardpoints absent/malformed unknown; `Pips[1] == 0` pass; `Pips[1] > 0` fail; missing/short `Pips` unknown; Cargo ship `Count > 0` pass; Cargo ship non-empty `Inventory` pass; Cargo ship empty count/inventory fail; non-Ship vessel unknown; malformed JSON unknown without panic.
  QA scenarios (name the exact tool + invocation): happy: `cargo test --all afk_checklist -- --nocapture` exits 0 and output is saved to `.omo/evidence/afk-checklist-watcher/task-2-afk-checklist-tests.txt`; failure: add/use a test fixture with malformed JSON and assert the resulting row state is `unknown`, not `fail` and not a panic, evidence `.omo/evidence/afk-checklist-watcher/task-2-malformed-json.txt`.
  Commit: Y | feat(runtime): add afk checklist companion parsing

- [x] 3. Expose `afk_checklist` in Rust AppSnapshot without breaking existing fields
  What to do / Must NOT do: Add `AfkChecklistView`/row types to the app export surface and add `pub afk_checklist: AfkChecklistView` to `AppSnapshot`. Update `AppSnapshot::from_state`, `snapshot::runtime_snapshot`, event-store bootstrap, and any test constructors to include a default unknown checklist until runtime state supplies parsed values. Do not remove or rename existing `session`, `missions`, `journal_source`, `matrix`, `web`, hull/shield/fighter fields.
  Parallelization: Wave 1 | Blocked by: 2 | Blocks: 4, 5, 9, 12
  References (executor has NO interview context - be exhaustive): `src/app/snapshot.rs:13-24` current snapshot fields; `src/app/snapshot.rs:27-45` snapshot construction; `src/app.rs:22-25` app DTO exports; `src/app/events.rs:81-89,99-106` event store snapshot construction/publication; `tests/runtime_service.rs:40-44` snapshot JSON privacy test.
  Acceptance criteria (agent-executable): `serde_json::to_string(&runtime.snapshot(now))` contains top-level key `afk_checklist`; existing `runtime_service_emits_sanitized_snapshot_and_notifications_from_fixture` still passes and still proves full local Journal paths are absent.
  QA scenarios (name the exact tool + invocation): happy: `cargo test --test runtime_service runtime_service_emits_sanitized_snapshot_and_notifications_from_fixture -- --nocapture` exits 0 and evidence is saved to `.omo/evidence/afk-checklist-watcher/task-3-runtime-snapshot.txt`; failure: add an assertion that `json.contains("afk_checklist")` is true and that existing `session.ship_hull_display` remains serialized, evidence `.omo/evidence/afk-checklist-watcher/task-3-snapshot-contract.txt`.
  Commit: Y | feat(app): expose afk checklist snapshot

- [x] 4. Initialize checklist from existing companion files on runtime startup
  What to do / Must NOT do: Extend `MonitorRuntime` to store checklist state and derive companion paths from `startup.journal_file.parent()`. During `MonitorRuntime::start`, perform one non-fatal read of `Status.json` and `Cargo.json` if present; missing/malformed files set affected rows to `unknown` and must not fail startup. Keep selected Journal preload behavior unchanged.
  Parallelization: Wave 2 | Blocked by: 2, 3 | Blocks: 7, 8, 12
  References (executor has NO interview context - be exhaustive): `src/app/runtime/service.rs:35-71` runtime startup and `LiveTail` creation; `src/app/runtime/service.rs:197-199` snapshot access; `tests/runtime_service.rs:8-43` runtime fixture style; `.omo/drafts/afk-checklist-watcher.md:50-54` scope includes companion parsing and immediate snapshots.
  Acceptance criteria (agent-executable): Add `cargo test --test runtime_service afk_checklist_initializes_from_existing_companion_files`, with temp directory containing selected Journal plus `Status.json` with `Flags: 64` and `Pips: [4,0,8]`, and `Cargo.json` with `Vessel: "Ship"`, `Count: 1`; snapshot has `hardpoints_deployed.state == "pass"`, `engine_pips_zero.state == "pass"`, `cargo_loaded.state == "pass"`. A second test with missing companion files asserts all three states are `unknown` and startup succeeds.
  QA scenarios (name the exact tool + invocation): happy: `cargo test --test runtime_service afk_checklist_initializes_from_existing_companion_files -- --nocapture` evidence `.omo/evidence/afk-checklist-watcher/task-4-startup-existing-files.txt`; failure: `cargo test --test runtime_service afk_checklist_missing_companion_files_are_unknown -- --nocapture` evidence `.omo/evidence/afk-checklist-watcher/task-4-missing-files.txt`.
  Commit: Y | feat(runtime): initialize afk checklist at startup

- [x] 5. Add runtime companion-update methods and Cargo journal-event fallback
  What to do / Must NOT do: Add a runtime method such as `MonitorRuntime::process_companion_update(path, now) -> Result<RuntimeStatusSnapshot or WatchCycle, RuntimeError>` or equivalent that rereads only the affected companion file, updates checklist state, and publishes a snapshot only when the derived `afk_checklist` value changes. Hook `JournalEvent::Cargo` inside the journal event processing path to non-fatally reread `Cargo.json` once after normal cargo event processing. Checklist-only changes must not produce terminal/Matrix notifications; warnings must be sanitized.
  Parallelization: Wave 2 | Blocked by: 2, 3 | Blocks: 7, 8, 12
  References (executor has NO interview context - be exhaustive): `src/app/runtime/service.rs:140-165` current `poll_once` publishes snapshots; `src/app/runtime/service.rs:201-210` `process_event` integration point; `src/app/events.rs:99-106` snapshot broadcast; `src/event.rs:517-523,1541-1555` Cargo event shape/parser; `.omo/drafts/afk-checklist-watcher.md:47,54` Cargo fallback and publish decisions.
  Acceptance criteria (agent-executable): Add `cargo test --test runtime_service afk_companion_update_publishes_checklist_only_snapshot` proving a Status/Cargo file update changes `snapshot.afk_checklist` without adding notifications. Add `cargo test --test runtime_service afk_cargo_journal_event_rereads_cargo_json_once` proving a missed Cargo.json watcher event is recovered when a Journal `Cargo` event is processed.
  QA scenarios (name the exact tool + invocation): happy: `cargo test --test runtime_service afk_companion_update_publishes_checklist_only_snapshot afk_cargo_journal_event_rereads_cargo_json_once -- --nocapture` evidence `.omo/evidence/afk-checklist-watcher/task-5-runtime-companion-update.txt`; failure: malformed `Cargo.json` after a Cargo event sets `cargo_loaded.state == "unknown"`, startup/runtime does not panic, and notifications stay unchanged, evidence `.omo/evidence/afk-checklist-watcher/task-5-cargo-malformed-fallback.txt`.
  Commit: Y | feat(runtime): process afk companion updates

- [x] 6. Add watcher abstraction with filtering, debounce, retry, and polling fallback tests
  What to do / Must NOT do: Introduce a small runtime watcher abstraction that owns `notify::RecommendedWatcher` and forwards normalized app-specific events through `tokio::sync::mpsc`. Watch selected file parent directory with `RecursiveMode::NonRecursive`. Filter to exact selected file path, `Status.json`, and `Cargo.json`; ignore unselected Journal files. Keep the watcher guard alive for the loop lifetime. If watcher initialization fails, emit one sanitized runtime warning and continue with existing polling fallback; do not crash watch mode. For companion files, coalesce duplicate events for the same path in a short debounce window and retry reads for likely partial writes before final `unknown`.
  Parallelization: Wave 2 | Blocked by: 1 | Blocks: 7, 8, 12
  References (executor has NO interview context - be exhaustive): `src/app/runtime/terminal.rs:60-64` terminal currently sleeps then polls; `src/app/runtime/desktop.rs:100-127` desktop currently sleeps then polls; `src/journal.rs:458-497` `LiveTail::poll` remains the Journal drain; `.omo/drafts/afk-checklist-watcher.md:25-27,45,48` directory-watch/filter/fallback decisions; `tests/runtime_service.rs:196-219` explicit non-Journal selected file behavior.
  Acceptance criteria (agent-executable): Tests prove: selected Journal path event is normalized; `Status.json` and `Cargo.json` events are normalized; unrelated `Journal.*.log` in same folder is ignored; explicit `--set-file fixture.log` still supports fallback polling; watcher init failure degrades to polling with one warning; partial JSON can be retried then accepted.
  QA scenarios (name the exact tool + invocation): happy: `cargo test --all afk_watcher -- --nocapture` evidence `.omo/evidence/afk-checklist-watcher/task-6-watcher-tests.txt`; failure: `cargo test --test runtime_service afk_watcher_ignores_unselected_journal_files afk_companion_update_retries_partial_json_then_publishes_valid_state -- --nocapture` evidence `.omo/evidence/afk-checklist-watcher/task-6-filter-retry.txt`.
  Commit: Y | feat(runtime): add afk file watcher adapter

- [x] 7. Integrate watcher-driven updates into the terminal watch loop
  What to do / Must NOT do: Replace the terminal `sleep(runtime.poll_interval()).await; poll_and_deliver(...)` loop with `tokio::select!` over watcher events and a `tokio::time::interval` fallback. The watcher branch must immediately process selected Journal append events through the existing `LiveTail::poll` path and companion events through the companion update method. The interval branch must keep existing warning/status housekeeping. If watcher unavailable, terminal loop must behave like current polling. Do not change replay mode.
  Parallelization: Wave 2 | Blocked by: 4, 5, 6 | Blocks: 12
  References (executor has NO interview context - be exhaustive): `src/app/runtime/terminal.rs:24-65` watch startup and loop; `src/app/runtime/terminal.rs:67-87` replay path to leave unchanged; `src/app/runtime/watch_runner.rs:65-90` reusable polling/delivery helpers; `src/journal.rs:432-433` poll interval source.
  Acceptance criteria (agent-executable): Add/adjust tests or a small integration harness proving terminal watch can receive a synthetic watcher Journal event and deliver a cycle without waiting for the interval; fallback interval still calls warning/status path; replay tests still pass.
  QA scenarios (name the exact tool + invocation): happy: `cargo test --all terminal_watch -- --nocapture` or the nearest implemented terminal watcher test exits 0, evidence `.omo/evidence/afk-checklist-watcher/task-7-terminal-watch.txt`; failure: `cargo test --test replay -- --nocapture` exits 0 to prove replay path was not affected, evidence `.omo/evidence/afk-checklist-watcher/task-7-replay-unchanged.txt`.
  Commit: Y | feat(runtime): drive terminal watch from file events

- [x] 8. Integrate watcher-driven updates into the desktop runtime loop without lock/await regressions
  What to do / Must NOT do: Update `DesktopRuntime` monitor task to use the same watcher event stream and fallback interval as terminal. Preserve the current lock/drop discipline: do not hold the `MonitorRuntime` mutex while awaiting delivery, and do not hold delivery mutex while doing file IO. Abort the monitor task on drop as before. If watcher initialization fails, desktop runtime continues polling and prints one sanitized warning.
  Parallelization: Wave 2 | Blocked by: 4, 5, 6 | Blocks: 12
  References (executor has NO interview context - be exhaustive): `src/app/runtime/desktop.rs:31-73` desktop startup; `src/app/runtime/desktop.rs:100-127` current monitor loop and lock/drop pattern; `src/app/runtime/watch_runner.rs:83-90` delivery helper; `.omo/drafts/afk-checklist-watcher.md:52-54` watcher scope and immediate snapshot publication.
  Acceptance criteria (agent-executable): Tests or compile-time structure show watcher branch creates a cycle then drops runtime lock before awaiting `deliver_watch_cycle`; `DesktopRuntime::snapshot()` returns checklist state; existing desktop/runtime tests continue passing.
  QA scenarios (name the exact tool + invocation): happy: `cargo test --all desktop -- --nocapture` or nearest implemented desktop runtime test exits 0, evidence `.omo/evidence/afk-checklist-watcher/task-8-desktop-watch.txt`; failure: a watcher-init-failure test asserts desktop runtime continues to produce snapshots from polling, evidence `.omo/evidence/afk-checklist-watcher/task-8-desktop-fallback.txt`.
  Commit: Y | feat(runtime): drive desktop watch from file events

- [x] 9. Update TypeScript snapshot schema, mocks, and normalization tests
  What to do / Must NOT do: Add `afkChecklistState` enum/schema and `afkChecklistViewSchema` to `ui/src/adapters/types.ts`; include `afk_checklist` in `appSnapshotSchema`; export inferred types as needed. Update `ui/src/adapters/mock-data.ts` with deterministic checklist data. Update `stableSnapshotKey()` to include `snapshot.afk_checklist`. Update adapter-boundary tests so generated_at/event_feed-only changes are ignored but checklist-only changes apply. Do not loosen Zod schema to `z.any()` and do not make `afk_checklist` optional in the final contract unless needed for a deliberate migration fallback.
  Parallelization: Wave 3 | Blocked by: 3 | Blocks: 10, 12
  References (executor has NO interview context - be exhaustive): `ui/src/adapters/types.ts:148-158` current strict app snapshot schema; `ui/src/adapters/mock-data.ts:3-195` mock snapshot; `ui/src/store/snapshot-normalization.ts:26-33` stable key currently omits checklist; `ui/e2e/adapter-boundary.spec.ts:120-137` current volatile-only/session-change test; `ui/src/adapters/web.ts:16-37,64-68` and `ui/src/adapters/tauri.ts:26-35,68-71` validate snapshots through schema.
  Acceptance criteria (agent-executable): `pnpm --dir ui typecheck` exits 0; `pnpm --dir ui test:e2e -- --grep "dashboard store|adapter"` exits 0; adapter-boundary includes a checklist-only snapshot update expectation returning true.
  QA scenarios (name the exact tool + invocation): happy: `pnpm --dir ui typecheck && pnpm --dir ui test:e2e -- --grep "dashboard store|adapter"` evidence `.omo/evidence/afk-checklist-watcher/task-9-ui-schema-normalization.txt`; failure: adapter test deliberately omits `afk_checklist` from a malformed payload and expects schema parsing to reject it, evidence `.omo/evidence/afk-checklist-watcher/task-9-schema-rejects-missing-checklist.txt`.
  Commit: Y | feat(ui): add afk checklist snapshot schema

- [x] 10. Replace Ship Integrity with Checklist panel and update dashboard e2e
  What to do / Must NOT do: In `TacticalTelemetryView`, replace the `Ship Integrity` panel with `Checklist`. Render the three required rows and status labels `PASS`, `FAIL`, `UNKNOWN` with semantic tones/classes consistent with existing tactical UI components. Remove unused `Shield`/`Meter` imports if no longer needed. Update smoke/e2e assertions to expect `Checklist`, row labels, and absence of `Ship Integrity`. Do not touch `reference-design/design1.tsx`.
  Parallelization: Wave 3 | Blocked by: 9 | Blocks: 12
  References (executor has NO interview context - be exhaustive): `ui/src/components/dashboard/tactical-telemetry-view.tsx:1-5` current imports; `ui/src/components/dashboard/tactical-telemetry-view.tsx:40-63` current `Ship Integrity` panel; `ui/e2e/dashboard-smoke.spec.ts:74-96` current operational regions test; project `AGENTS.md` says frontend/Tauri/WebUI changes require Windows package rebuild.
  Acceptance criteria (agent-executable): Dashboard renders region `Checklist`; region contains `Hardpoints deployed`, `Engine pips zero`, `Cargo loaded`; `Ship Integrity` region count is 0; e2e screenshots update under `.omo/evidence/afk-checklist-watcher/`.
  QA scenarios (name the exact tool + invocation): happy: `pnpm --dir ui test:e2e -- --grep "operational regions|Checklist"` evidence `.omo/evidence/afk-checklist-watcher/task-10-checklist-e2e.txt`; failure: e2e assertion `await expect(page.getByRole("region", { name: "Ship Integrity" })).toHaveCount(0)` catches stale UI, evidence `.omo/evidence/afk-checklist-watcher/task-10-no-ship-integrity.txt`.
  Commit: Y | feat(ui): replace ship integrity with checklist

- [x] 11. Update user-facing docs/knowledge only for behavior changes
  What to do / Must NOT do: Update README or a focused `.omo/knowledges/*.md` note if the final implementation changes user-visible behavior around watch latency, checklist semantics, or package verification. Keep docs privacy-safe: no raw local Journal paths, no private commander/system names, no tokens. Do not document unsupported cargo-value estimation or journal auto-rotation.
  Parallelization: Wave 4 | Blocked by: 1-10 | Blocks: 12
  References (executor has NO interview context - be exhaustive): `README.md` watch-mode and package sections; `.omo/knowledges/journal-status-ship-state-capabilities-2026-06-26.md` existing ED source findings; project `AGENTS.md` knowledge-saving rule; `README.md` privacy section warns against raw local Journals.
  Acceptance criteria (agent-executable): If docs changed, they state `poll_interval_ms` remains fallback/housekeeping, `Checklist` uses tri-state values, and cargo readiness is non-empty ship cargo; docs do not mention or expose raw private file paths. If no user-facing docs change is needed, add a short `.omo/knowledges/afk-checklist-implementation-notes-2026-06-26.md` implementation note.
  QA scenarios (name the exact tool + invocation): happy: `rg -n "Checklist|afk_checklist|poll_interval_ms|Cargo.json|Status.json" README.md .omo/knowledges` evidence `.omo/evidence/afk-checklist-watcher/task-11-docs-search.txt`; failure/privacy: `rg -n --hidden --glob '!target/**' --glob '!ui/node_modules/**' --glob '!ui/dist/**' --glob '!dist/**' 'access_token\s*=\s*"[^"<][^"]{8,}"|BEGIN (RSA|OPENSSH|PRIVATE) KEY|/home/.*/Elite Dangerous|Journal\.[0-9].*\.log' README.md .omo/knowledges ui/src src tests` returns no private hits except committed synthetic fixture/test names already allowed by repo policy, evidence `.omo/evidence/afk-checklist-watcher/task-11-privacy-scan.txt`.
  Commit: Y | docs: record afk checklist watcher behavior

- [x] 12. Run final automated verification and rebuild Windows GNU artifact
  What to do / Must NOT do: Run the complete automated gate after all implementation/docs todos are complete. Save command output under `.omo/evidence/afk-checklist-watcher/`. Do not skip packaging because frontend/Tauri/WebUI asset files changed. Do not report completion if any command fails.
  Parallelization: Wave 4 | Blocked by: 1-11 | Blocks: final delivery
  References (executor has NO interview context - be exhaustive): `README.md` verification commands; `ui/package.json:7-18` UI scripts; `scripts/package-windows-gnu.sh:103-109` package outputs/hashes; project `AGENTS.md` packaging rule.
  Acceptance criteria (agent-executable): All final commands exit 0; package artifacts exist; package output includes expected SHA-256 lines; generated evidence files are present.
  QA scenarios (name the exact tool + invocation): happy: run `cargo fmt --check`, `cargo metadata --format-version 1 >/tmp/ed-sentry-cargo-metadata.json`, `cargo test --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `pnpm --dir ui lint`, `pnpm --dir ui typecheck`, `pnpm --dir ui build`, `pnpm --dir ui test:e2e -- --grep "Checklist|dashboard store|adapter|operational regions"`, and `./scripts/package-windows-gnu.sh`, saving outputs to `.omo/evidence/afk-checklist-watcher/task-12-final-gates.txt`; failure: if a gate fails, save the failing command output to `.omo/evidence/afk-checklist-watcher/task-12-failure-<gate>.txt`, fix the implementation in the responsible earlier todo, then rerun the full gate.
  Commit: Y | chore: verify afk checklist watcher release build

## Final verification wave

> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.

- [x] F1. Plan compliance audit
  - Verify every Must Have is implemented, every Must NOT is absent, and no product code changed outside the planned surfaces without justification.
  - Evidence: `.omo/evidence/afk-checklist-watcher/f1-plan-compliance.md`.
- [x] F2. Code quality review
  - Review Rust/TypeScript for minimality, strict typing, no panics/unwraps in new runtime paths, no duplicated parsing logic, and no UI schema loosening.
  - Evidence: `.omo/evidence/afk-checklist-watcher/f2-code-quality.md`.
- [x] F3. Automated runtime/UI QA
  - Re-run the final command gates from Todo 12 and confirm evidence paths and artifacts exist.
  - Evidence: `.omo/evidence/afk-checklist-watcher/f3-automated-qa.md`.
- [x] F4. Scope fidelity/privacy audit
  - Verify no Journal auto-rotation, cargo market-value estimation, game automation, new watcher config flags, raw private Journal paths, or secrets were introduced.
  - Evidence: `.omo/evidence/afk-checklist-watcher/f4-scope-privacy.md`.

## Commit strategy

- Prefer small reviewable commits matching todo boundaries where practical:
  1. dependency/parser/snapshot foundation,
  2. runtime watcher integration,
  3. frontend schema/UI/tests,
  4. docs/final verification artifacts.
- Before any commit, inspect `git status`, `git diff`, and `git log --oneline -10`; stage only intended files; never stage local secrets/configs/raw Journals.
- Do not commit generated build output unless this repository normally tracks that artifact. The required Windows package rebuild is a reporting/verification artifact unless the worker confirms tracked files changed intentionally.
- Never use `git clean`, force push, amend, or skip hooks unless the user explicitly requests it.

## Success criteria

- `AppSnapshot` includes top-level `afk_checklist` with exactly the three planned rows and tri-state states.
- Runtime reads existing `Status.json`/`Cargo.json` on startup and updates checklist state through watcher events without waiting for `poll_interval_ms`.
- Selected Journal appends are watcher-triggered through the existing `LiveTail::poll()` path; polling remains as fallback/housekeeping.
- Missing/malformed/unreadable companion files produce `unknown`, not crashes or false pass/fail values.
- `JournalEvent::Cargo` rereads `Cargo.json` once as a fallback for missed Cargo watcher events.
- UI shows a `Checklist` panel and no `Ship Integrity` region; rows show `Hardpoints deployed`, `Engine pips zero`, and `Cargo loaded` with `PASS`/`FAIL`/`UNKNOWN`.
- Browser and Tauri adapters parse the new snapshot schema; checklist-only snapshots are applied by the store.
- All Rust/UI/e2e/package gates listed in Verification strategy pass.
- Windows GNU package is regenerated and the script prints expected SHA-256 lines.
- No scope creep: no cargo market value estimation, no Journal auto-rotation, no raw path leaks, no new watcher config, no game automation.
