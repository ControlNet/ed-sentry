# gui-webui-tauri - Work Plan

## TL;DR (For humans)
**What you'll get:** A shared live dashboard for `ed-sentry` that works in the browser and in a separate desktop GUI. The existing CLI remains the core executable, and when config enables WebUI it serves a local full-stack dashboard while continuing normal monitoring.

**Why this approach:** The monitor already has a strong Rust core, so the plan extracts a shared application service instead of duplicating watch logic for CLI, Web, and desktop. The frontend is one React/Vite/shadcn app with different adapters for browser WebUI and Tauri.

**What it will NOT do:** No GUI replay, no historical database, no public/authenticated remote mode, no chart library in the first milestone, and no Matrix command/automation work.

**Effort:** XL
**Risk:** High - this crosses runtime orchestration, config/security, WebSocket transport, frontend architecture, and desktop packaging.
**Decisions to sanity-check:** WebUI starts from `[web] enabled = true` rather than a CLI flag; config editing is in first milestone; Tauri must honor config-enabled Web/Matrix services instead of being a frontend-only wrapper.

Your next move: approve starting work from this plan, or ask for a high-accuracy review first. Full execution detail follows below.

---

> TL;DR (machine): XL/high risk; build shared Rust app service, Axum WebUI, React/Vite/shadcn dashboard, config editing, backend event buffer, and Tauri `ed-sentry-gui` entry.

## Scope
### Must have
- Root `DESIGN.md` before any React/shadcn component work, with a dark operational dashboard design system specific to `ed-sentry`.
- Shared frontend under `ui/` using React + Vite + TypeScript + pnpm + Tailwind + shadcn/ui + lucide-react + Zustand.
- Rust application service layer that moves watch orchestration out of `src/main.rs` into reusable library code for CLI, WebUI, and Tauri.
- `[web]` config section with `enabled`, `host`, `port`, and `open_browser`; default disabled, local-first, and no separate `--webui` flag.
- Axum WebUI backend compiled into `ed-sentry` by default and started in watch-capable runtimes only when `[web] enabled = true`.
- HTTP endpoints for sanitized snapshot/config views and config updates.
- WebSocket realtime stream for current snapshot, recent backend event buffer, live notifications, status updates, warnings, and connection lifecycle events.
- Backend fixed-size recent event ring buffer so late-opening WebUI/Tauri views can see previous current-process events.
- GUI config editing for relevant config fields, including safe Matrix token handling that never echoes raw token values back to the frontend.
- Shared frontend adapter boundary so the same pages work in browser WebUI and Tauri.
- `ed-sentry-gui` desktop entry that bootstraps the same Rust services and honors config-enabled Web/Matrix behavior where supported.
- First milestone live dashboard: session health, commander/ship/system, kill/scan/bounty/merits metrics, shield/hull/fighter/fuel state, warnings, event feed, mission progress, Journal source, Matrix status, WebUI status, and config editing.
- Docs, tests, real browser smoke/visual QA, and security checks for config/token boundaries.

### Must NOT have (guardrails, anti-slop, scope boundaries)
- No GUI replay. Existing CLI `--replay` remains terminal-only and does not initialize WebUI or Matrix.
- No historical database, multi-day analytics, auth/public remote mode, or chart library in the first milestone.
- No Matrix command handling, encrypted Matrix rooms, Discord/Telegram, EDMC, auto relog, key simulation, game automation, or dashboard builder.
- No raw Matrix access token in API responses, logs, frontend state, fixtures, screenshots, docs, or committed config.
- No raw local Journal lines or private commander/chat content in frontend fixtures or evidence artifacts.
- No generic shadcn admin-template look; shadcn/ui is source scaffolding only and must be rethemed through `DESIGN.md`.
- No duplicate watch loops for terminal/Web/Tauri. There must be one monitor pipeline fanning out to sinks/subscribers.
- No breaking the existing no-subcommand CLI contract: default remains watch mode; `--replay` remains replay mode.

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: TDD for Rust behavior/config/service seams; tests-after plus Playwright smoke/visual QA for UI once the frontend exists.
- Rust gates: `cargo fmt --check`, `cargo test --all`, `cargo clippy --all-targets --all-features -- -D warnings`.
- Frontend gates: `pnpm --dir ui install --frozen-lockfile`, `pnpm --dir ui typecheck`, `pnpm --dir ui build`, `pnpm --dir ui lint` if lint is configured, and Playwright browser smoke tests.
- WebUI surface gates: run `scripts/smoke-webui.sh` scenarios against a temp config containing `[web] enabled = true`, hit HTTP endpoints with `curl -i`, connect WebSocket with `scripts/probe-websocket.mjs`, and verify snapshot/event/config behavior from observable payloads.
- Desktop surface gates: run `pnpm --dir ui tauri build` in a desktop-capable environment when available; otherwise record the concrete Tauri system-dependency limitation plus browser-equivalent frontend QA.
- Evidence path pattern: `.omo/evidence/gui-webui-tauri/task-<N>-<slug>.<ext>`.

## Review hardening decisions
- Config editing must introduce an explicit `ConfigSource`/`ConfigPath` model before any write endpoint is implemented. CLI WebUI writes to explicit `--config <file>` when provided, otherwise to `./config.toml`; if no implicit file exists, the first successful save creates `./config.toml` from sanitized defaults plus edited values. Tauri uses the platform app config directory for `ed-sentry/config.toml` when no explicit config path is provided. Malformed or unreadable config disables writes until the error is fixed; permission failures return frontend-safe errors.
- Config writes must use a comment/unknown-key preserving TOML edit path, for example `toml_edit`, for existing files. New files may be generated from a safe template. Tests must prove known fields update, unknown keys remain, comments are preserved where the chosen TOML tool supports it, and raw Matrix tokens are never echoed.
- First milestone WebUI mutation endpoints are loopback-only. Non-loopback WebUI binds may serve read-only status/snapshot endpoints with a warning, but `PUT /api/config` and any future state-changing endpoint must return `403` unless a later authenticated remote mode is explicitly designed. Validate `Host` and `Origin`, use restrictive CORS, and do not use wildcard CORS for state-changing routes.
- First milestone WebUI assets are not embedded into the Rust binary. `ed-sentry` serves a built asset directory from this lookup order: `ED_SENTRY_WEBUI_DIST` for tests/dev overrides, sibling `webui/` beside the executable for packaged artifacts, then repo-local `ui/dist` for development. Release packaging must copy `ui/dist` to `webui/` and verify the packaged binary serves `/`.
- Tauri layout is `ui/src-tauri/` using Tauri v2, with the shared Vite frontend in `ui/`. Add a Cargo workspace if required, make the Tauri crate depend on the root `ed-sentry` library by path, set the desktop binary/product name to `ed-sentry-gui`, and expose desktop-only behavior through adapter code rather than copied pages.
- Web/backend dependency target: add Axum with WebSocket support, `tower-http` for static files/CORS/trace as needed, and required `tokio` features such as `net`, `fs`, and `sync`. Add `toml_edit` for config writes and a warning-only browser-opening crate only if `open_browser` is implemented.
- Frontend QA tooling is established in the scaffold task, not deferred to visual QA. The first UI scaffold must add Playwright config, `@playwright/test`, scripts for `test:e2e` and Chromium install, and screenshot output under `.omo/evidence/gui-webui-tauri/`.
- Smoke/probe scripts must exist before plan steps depend on them: `scripts/smoke-webui.sh` for temp config/port/process lifecycle/HTTP probes, `scripts/probe-websocket.mjs` for `/api/events`, and `scripts/verify-gui-webui-tauri.sh` for the final gate. These scripts must not require `websocat`.

## Execution strategy
### Parallel execution waves
> Target 5-8 todos per wave. Fewer than 3 (except the final) means you under-split.
- Wave 1: Foundation and contracts: design system, web config schema, app-service DTO contracts, frontend scaffold, and service characterization tests.
- Wave 2: Shared runtime and Web backend: application service extraction, event buffer, Axum server, HTTP/WebSocket APIs, config update path.
- Wave 3: Frontend product surface: adapters, dashboard, event feed, mission/config panels, visual system, WebUI browser QA.
- Wave 4: Desktop entry and integration: Tauri `ed-sentry-gui`, shared frontend adapter, service bootstrap parity, packaging/docs.
- Wave 5: Hardening and release readiness: security audit, full verification, visual QA, docs, final gate.

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1 | none | 5, 9, 10, 11, 12 | 2, 3, 4 |
| 2 | none | 6, 8, 13 | 1, 3, 4 |
| 3 | none | 6, 7, 8, 13 | 1, 2, 4 |
| 4 | none | 9, 10, 11, 12 | 1, 2, 3 |
| 5 | 1, 3 | 6, 7, 8, 13, 15 | none |
| 6 | 2, 3, 5 | 7, 8, 13, 15 | 9 |
| 7 | 3, 5, 6 | 8, 10, 12, 13 | 9 |
| 8 | 2, 3, 6, 7 | 10, 11, 13, 15 | 9 |
| 9 | 1, 4 | 10, 11, 12 | 6, 7 |
| 10 | 1, 7, 8, 9 | 11, 12, 14 | none |
| 11 | 1, 8, 9, 10 | 14 | 12 |
| 12 | 1, 7, 9, 10 | 14 | 11 |
| 13 | 2, 3, 5, 6, 7, 8 | 14, 15 | none |
| 14 | 10, 11, 12, 13 | 15 | none |
| 15 | 5, 6, 8, 13, 14 | final verification | none |

## Todos
> Implementation + Test = ONE todo. Never separate.
<!-- APPEND TASK BATCHES BELOW THIS LINE WITH edit/apply_patch - never rewrite the headers above. -->
- [x] 1. Create the root design system and frontend token contract

  What to do: Create `DESIGN.md` at the repository root before writing UI code. Use a dark operational dashboard identity for an Elite Dangerous AFK monitor: dense, low-glare, technical, status-first, no marketing hero. Define color tokens, typography, spacing/layout, depth strategy, motion, and initial component patterns for dashboard panels, metric tiles, event feed, mission table, status badges, config forms, and shell navigation. Include shadcn/ui usage rules that explicitly retheme default components and forbid generic admin-template visuals.

  Must NOT do: Do not create React components in this task. Do not use emojis as UI icons. Do not introduce raw token-free colors or magic spacing values. Do not make the UI a landing page.

  Parallelization: Wave 1 | Blocked by: none | Blocks: 5, 9, 10, 11, 12

  References (executor has NO interview context - be exhaustive):
  - `.omo/knowledges/gui-readiness-2026-06-20.md` - confirmed design direction, stack, shadcn decision, and dashboard scope.
  - `README.md` - product scope, CLI behavior, config privacy, Matrix boundaries, and current non-goals.
  - `omo:frontend` skill reference `references/design/design-system-architecture.md` - required 7-section `DESIGN.md` structure.
  - `ui-ux-pro-max` skill guidance - project instruction prefers it for UI/UX design work; use it to validate dashboard style choices before coding components.
  - `config.example.toml` - current config vocabulary and user-facing defaults.

  Acceptance criteria (agent-executable):
  - `test -f DESIGN.md` exits `0`.
  - `rg -n "shadcn|Event Feed|Mission|Metric|Matrix|Journal|token|Status" DESIGN.md` finds the expected design-system coverage.
  - `rg -n "#[0-9A-Fa-f]{6}" DESIGN.md` returns only palette token rows, not ad-hoc component prose.
  - No files under `ui/` are created by this task.

  QA scenarios:
  - Happy path: `sed -n '1,260p' DESIGN.md > .omo/evidence/gui-webui-tauri/task-1-design-system.txt`; expected output contains all seven sections and dashboard-specific component patterns.
  - Failure edge: `rg -n "hero|landing|generic admin|emoji" DESIGN.md > .omo/evidence/gui-webui-tauri/task-1-design-system-guard.txt`; expected matches are absent or only appear in explicit anti-pattern language.

  Commit: YES | `docs(gui): define dashboard design system`

- [x] 2. Add `[web]` config schema, defaults, docs, and privacy tests

  What to do: Extend `src/config.rs` with `WebConfig` and carry it through `AppConfig` and `RuntimeConfig`. Parse `[web]` with the same manual `toml::Value` style as `[matrix]`. Defaults: `enabled = false`, `host = "127.0.0.1"`, `port = 8765`, `open_browser = false`. Add warnings for wrong-typed keys. Add safe local-first warning behavior for non-localhost bind addresses, but do not block startup. Introduce `ConfigSource`/`ConfigPath` metadata for future writes: explicit CLI config path, implicit `./config.toml`, defaults-only with first-save target, and Tauri app-config target. Update `config.example.toml` and README config docs. Ensure replay ignores WebUI startup by design.

  Must NOT do: Do not add a `--webui` flag. Do not make `[web]` enabled by default. Do not expose or modify Matrix token behavior. Do not start any server in this task.

  Parallelization: Wave 1 | Blocked by: none | Blocks: 6, 8, 13

  References:
  - `src/config.rs` - `AppConfig`, `RuntimeConfig`, `MatrixConfig`, `from_toml_str`, `load_optional`, `into_runtime`, and manual section parsing patterns.
  - `tests/cli_config.rs` - binary/config integration patterns.
  - `config.example.toml` - committed safe template.
  - `README.md` - config precedence and current replay/Matrix isolation docs.
  - `.omo/knowledges/gui-readiness-2026-06-20.md` - confirmed `[web] enabled` startup decision.

  Acceptance criteria:
  - New unit tests prove default disabled config, enabled parsing, CLI/runtime carry-through, wrong-type warnings, and non-localhost warning text.
  - New tests prove config source metadata for explicit `--config`, existing implicit `./config.toml`, absent config first-save target, malformed strict config, permission/read failure, and Tauri app-config target.
  - `cargo test --lib config::tests::config_web_defaults_to_disabled_localhost` exits `0`.
  - `cargo test --lib config::tests::config_source_tracks_write_target` exits `0`.
  - `cargo test --test cli_config cli_config_web_section_is_accepted_without_starting_server` exits `0`.
  - `cargo test --all` exits `0`.
  - `README.md` and `config.example.toml` document `[web]` without documenting any `--webui` flag.

  QA scenarios:
  - Happy path: `cargo test --lib config::tests::config_web_enabled_preserves_host_port_open_browser > .omo/evidence/gui-webui-tauri/task-2-web-config.txt`; expected test exits `0` and asserts parsed runtime fields.
  - Failure edge: `cargo test --lib config::tests::config_web_wrong_typed_keys_warn_and_keep_defaults > .omo/evidence/gui-webui-tauri/task-2-web-config-warnings.txt`; expected test exits `0` and asserts defaults remain safe.

  Commit: YES | `feat(config): add web ui settings`

- [x] 3. Define shared app-service view models and config-edit DTOs

  What to do: Add a library module such as `src/app.rs` or `src/app/mod.rs` exported from `src/lib.rs`. Define DTOs for `AppSnapshot`, `SessionView`, `MissionView`, `NotificationView`, `EventFeedItem`, `JournalSourceView`, `MatrixStatusView`, `WebStatusView`, and sanitized editable config views. Derive `Serialize`, `Deserialize` only where needed for Web/Tauri transport. Include raw values plus display text for CLI-parity-sensitive values such as rates, durations, timestamps, bounty totals, and status labels. Add conversion functions from `SessionState`, `MissionTracker`, `Notification`, `RuntimeConfig`, and Matrix/Web startup statuses.

  Must NOT do: Do not move the watch loop yet. Do not serialize raw `MatrixConfig.access_token` to any frontend DTO. Do not expose raw Journal line contents. Do not make DTOs depend on frontend TypeScript names.

  Parallelization: Wave 1 | Blocked by: none | Blocks: 6, 7, 8, 13

  References:
  - `src/lib.rs` - module export pattern.
  - `src/state.rs` - `SessionState` public fields and rate helpers.
  - `src/mission.rs` - `MissionTracker`, `TrackedMission`, `MissionProgress`.
  - `src/notifier.rs` - `Notification`, `AlertLevel`, `mention`.
  - `src/terminal.rs` - existing display formatting helpers for status/rates.
  - `src/config.rs` - Matrix token redaction behavior and runtime config.
  - `.omo/knowledges/mission-modeling-odelitetracker.md` - dashboard mission modeling boundary.
  - `.omo/knowledges/typed-journal-raw-payload-preservation.md` - raw payload preservation implications.

  Acceptance criteria:
  - `src/lib.rs` exports the new app/service module.
  - Unit tests prove snapshot conversion from a synthetic `SessionState` and `MissionTracker`.
  - Unit tests prove sanitized config DTO omits raw Matrix token while preserving token-present/absent state.
  - `cargo test --lib app` exits `0`.
  - `cargo test --all` exits `0`.

  QA scenarios:
  - Happy path: `cargo test --lib app::tests::app_snapshot_includes_session_mission_and_display_values > .omo/evidence/gui-webui-tauri/task-3-app-dto.txt`; expected test exits `0` and asserts core dashboard fields.
  - Failure edge: `cargo test --lib app::tests::config_view_redacts_matrix_access_token > .omo/evidence/gui-webui-tauri/task-3-token-redaction.txt`; expected test exits `0` and asserts token value is absent from serialized JSON.

  Commit: YES | `feat(app): add dashboard view models`

- [x] 4. Scaffold the shared React/Vite/shadcn frontend under `ui/`

  What to do: Create `ui/` with pnpm, React + Vite + TypeScript, Tailwind, shadcn/ui, lucide-react, Zustand, Playwright, and baseline scripts: `dev`, `build`, `preview`, `typecheck`, `test:e2e`, `test:e2e:install`, and `lint` if lint is configured. Configure Tailwind/shadcn tokens to map to `DESIGN.md`. Add React dev tooling gates only in development if required by frontend skill guidance. Create adapter interfaces but keep implementations mocked until backend APIs exist. Add `playwright.config.ts` with Chromium project and screenshot output under `.omo/evidence/gui-webui-tauri/`.

  Must NOT do: Do not create a marketing landing page. Do not hardcode production API URLs. Do not commit generated `node_modules`. Do not use default shadcn visual identity without retheming.

  Parallelization: Wave 1 | Blocked by: none | Blocks: 9, 10, 11, 12

  References:
  - Official Vite docs for React + TypeScript setup: https://vite.dev/guide/
  - Official shadcn/ui Vite installation docs: https://ui.shadcn.com/docs/installation/vite
  - `omo:frontend` skill React dev tooling gate.
  - `DESIGN.md` from Todo 1.
  - `.gitignore` - add Node/Tauri generated ignores without removing existing Rust ignores.

  Acceptance criteria:
  - `test -f ui/package.json` exits `0`.
  - `pnpm --dir ui install --frozen-lockfile` exits `0`.
  - `pnpm --dir ui typecheck` exits `0`.
  - `pnpm --dir ui build` exits `0`.
  - `pnpm --dir ui test:e2e:install` installs Chromium or records a concrete browser-install environment blocker.
  - `pnpm --dir ui test:e2e -- --project=chromium` exits `0` for the scaffold smoke test.
  - shadcn base components exist under `ui/src/components/ui/`.
  - `rg -n "TODO|lorem|emoji|🚀|✨" ui/src` returns no placeholder UI content.

  QA scenarios:
  - Happy path: `pnpm --dir ui build > .omo/evidence/gui-webui-tauri/task-4-ui-build.txt`; expected build exits `0` and writes `ui/dist`.
  - Failure edge: `pnpm --dir ui typecheck > .omo/evidence/gui-webui-tauri/task-4-ui-typecheck.txt`; expected typecheck exits `0` with no implicit-any or missing module errors.
  - Browser tooling: `pnpm --dir ui test:e2e -- --project=chromium > .omo/evidence/gui-webui-tauri/task-4-playwright.txt`; expected scaffold smoke exits `0` or records only a concrete browser-install blocker.

  Commit: YES | `feat(ui): scaffold shared dashboard frontend`

- [x] 5. Extract a reusable monitor runtime service from `src/main.rs`

  What to do: Move watch-mode orchestration into reusable library code, for example `src/app/runtime.rs`, while keeping CLI behavior unchanged. The service should select/preload the Journal file, drive one `LiveTail`, own one `EventMonitor` and one `MissionTracker`, run warnings/status ticks, and fan out notifications/status snapshots to terminal, Matrix, and future Web/Tauri subscribers. Keep `src/main.rs` as CLI parsing, banner/startup display, and entrypoint wiring. Add characterization tests before refactor that pin current CLI replay/watch behavior.

  Must NOT do: Do not change replay output. Do not create a second watch loop for WebUI. Do not remove Matrix best-effort behavior. Do not weaken existing CLI tests.

  Parallelization: Wave 2 | Blocked by: 1, 3 | Blocks: 6, 7, 8, 13, 15

  References:
  - `src/main.rs` - current `run_watch`, `run_replay`, Matrix delivery connection, startup printing, status rendering, and delivery helper functions.
  - `src/journal.rs` - `preload_journal_file`, `LiveTail::from_preload`, `LiveTail::poll`, `live_poll_interval`.
  - `src/monitor.rs` - producer API and status rendering.
  - `src/mission.rs` - mission tracker for dashboard state.
  - `src/delivery.rs` - `DeliveryHub`, `RemoteDelivery`, status cadence.
  - `tests/cli_config.rs`, `tests/replay.rs`, `tests/live_tail.rs` - characterization tests.

  Acceptance criteria:
  - Existing `cargo test --test replay` passes unchanged or with intentional path-only updates.
  - Existing `cargo test --test cli_config cli_config_watch_tails_until_stopped` passes.
  - New unit/integration tests prove the service emits snapshots and notifications from sanitized fixtures.
  - `rg -n "LiveTail::from_preload|EventMonitor::from_runtime_config" src/main.rs` shows orchestration moved out of CLI entrypoint except thin wiring if unavoidable.
  - `cargo test --all` exits `0`.

  QA scenarios:
  - Happy path: `cargo test --test live_tail live_tail_temp_file_drives_monitor_notifier_pipeline_without_sleeping > .omo/evidence/gui-webui-tauri/task-5-runtime-live-tail.txt`; expected test exits `0` and proves one pipeline processes appended events.
  - Failure edge: `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line > .omo/evidence/gui-webui-tauri/task-5-replay-regression.txt`; expected exit `0` and output contains `Scan`, `Kill`, `Total Stats`, and `Monitor stopped`.

  Commit: YES | `refactor(app): extract monitor runtime service`

- [x] 6. Add backend recent-event buffer and broadcast channels

  What to do: Implement a backend event store in the app service: fixed-size ring buffer, snapshot channel for current state, and broadcast channel for live updates. Store sanitized `EventFeedItem`/`NotificationView` records, status updates, warnings, and lifecycle events. Ensure new subscribers receive current snapshot plus recent buffered events before live updates. Use deterministic tests with a small buffer size to prove eviction order.

  Must NOT do: Do not persist events to disk or add a database. Do not store raw Journal lines or Matrix tokens. Do not make frontend memory the source of truth.

  Parallelization: Wave 2 | Blocked by: 2, 3, 5 | Blocks: 7, 8, 13, 15

  References:
  - New app-service module from Todo 3 and Todo 5.
  - `src/notifier.rs` - `Notification` fields and level filtering semantics.
  - `src/text.rs` - `line_safe` sanitation.
  - `.omo/knowledges/gui-readiness-2026-06-20.md` - backend event buffer decision.

  Acceptance criteria:
  - Unit tests prove ring buffer preserves latest N events and evicts oldest.
  - Unit tests prove new subscriber bootstrap payload contains snapshot plus buffered events.
  - Unit tests prove `line_safe` is applied to event text exposed to frontend.
  - `cargo test --lib app::tests::event_buffer` exits `0`.

  QA scenarios:
  - Happy path: `cargo test --lib app::tests::new_subscriber_receives_snapshot_and_recent_events > .omo/evidence/gui-webui-tauri/task-6-event-buffer.txt`; expected test exits `0`.
  - Failure edge: `cargo test --lib app::tests::event_buffer_evicts_oldest_without_raw_journal_text > .omo/evidence/gui-webui-tauri/task-6-event-buffer-eviction.txt`; expected test exits `0`.

  Commit: YES | `feat(app): buffer dashboard events`

- [x] 7. Implement the Axum WebUI server, smoke harness, and packaged static asset lookup

  What to do: Add a `src/web.rs` module using Axum. Start it only when runtime config has `[web] enabled = true`, in watch-capable app services. Bind to configured host/port; port `0` may use OS-assigned port if implemented. Add dependencies with required features: `axum` with WebSocket support, `tower-http` for static file/CORS/trace behavior as needed, required `tokio` features such as `net`, `fs`, and `sync`, and a warning-only browser-opening crate only if `open_browser` is implemented. Serve static files from the phase-one asset lookup: `ED_SENTRY_WEBUI_DIST`, sibling `webui/` beside executable, then repo-local `ui/dist`. Add `scripts/smoke-webui.sh` to create a temp config, choose a free local port, start/stop `ed-sentry`, export/print the resolved URL, run HTTP probes, and write evidence. WebUI startup failure should return a warning and allow monitor runtime to continue. Non-localhost bind should emit a security warning and disable state-changing endpoints.

  Must NOT do: Do not start WebUI in replay. Do not add auth/public remote mode. Do not crash watch because the Web server fails to bind. Do not serve source files from `ui/src`. Do not require `websocat`.

  Parallelization: Wave 2 | Blocked by: 3, 5, 6 | Blocks: 8, 10, 12, 13

  References:
  - Official Axum WebSocket extractor docs: https://docs.rs/axum/latest/axum/extract/ws/index.html
  - `src/main.rs` / app runtime from Todo 5 - startup integration point.
  - `src/config.rs` - `WebConfig`.
  - `src/delivery.rs` - warning style and best-effort sink precedent.
  - `README.md` - startup messaging style.

  Acceptance criteria:
  - `cargo test --lib web` exits `0`.
  - `cargo check --all-targets --all-features` exits `0` after dependency/feature changes.
  - `cargo tree -i axum` and `cargo tree -i tower-http` show the intended web dependencies.
  - Integration test starts WebUI against temp `ED_SENTRY_WEBUI_DIST` and `curl`/HTTP client receives `200` for `/`.
  - Integration test proves sibling `webui/` lookup works from a temp packaged binary directory.
  - Integration test binds an occupied port and proves watch runtime continues with warning.
  - Integration test proves replay path does not start WebUI even when `[web] enabled = true`.
  - `test -x scripts/smoke-webui.sh` exits `0`.
  - `cargo test --all` exits `0`.

  QA scenarios:
  - Happy path HTTP: `scripts/smoke-webui.sh --scenario root --evidence .omo/evidence/gui-webui-tauri/task-7-web-root.http`; expected status line is `HTTP/1.1 200 OK` and body contains the app root.
  - Packaged asset lookup: `scripts/smoke-webui.sh --scenario packaged-assets --evidence .omo/evidence/gui-webui-tauri/task-7-packaged-assets.http`; expected packaged sibling `webui/` serves `/`.
  - Failure edge: `scripts/smoke-webui.sh --scenario occupied-port --evidence .omo/evidence/gui-webui-tauri/task-7-web-port-warning.txt`; expected warning mentions WebUI bind failure and monitor startup continues.

  Commit: YES | `feat(web): serve embedded dashboard app`

- [x] 8. Add HTTP APIs, WebSocket protocol, config-write safety, and probe script

  What to do: Implement HTTP endpoints for sanitized current snapshot, sanitized config view, config update, WebUI status, Matrix status, and health. Implement WebSocket endpoint that sends initial snapshot plus recent event buffer, then live updates. Define explicit JSON message envelopes with version/type fields. Add frontend-safe error shapes. Config update should validate and write through backend logic, preserving or replacing Matrix access token only according to explicit frontend input. Use the `ConfigSource`/`ConfigPath` model from Todo 2. For first milestone, allow config mutation only on loopback-bound WebUI servers; reject state-changing routes on non-loopback binds with `403`. Validate `Host`/`Origin`, configure restrictive CORS, and add `scripts/probe-websocket.mjs` using project Node dependencies rather than `websocat`.

  Must NOT do: Do not send raw `config.toml` contents. Do not echo Matrix access token. Do not accept path traversal/static file escapes. Do not allow frontend to enable unsafe remote binding without backend warning state. Do not allow unauthenticated config mutation from LAN/non-loopback origins.

  Parallelization: Wave 2 | Blocked by: 2, 3, 6, 7 | Blocks: 10, 11, 13, 15

  References:
  - App DTO module from Todo 3.
  - Event buffer from Todo 6.
  - Axum server from Todo 7.
  - `src/config.rs` - config parse/write patterns; add write helpers if missing.
  - `tests/cli_config.rs` - temp config patterns.
  - `.omo/knowledges/gui-readiness-2026-06-20.md` - config editing decision.

  Acceptance criteria:
  - `GET /api/snapshot` returns sanitized dashboard JSON.
  - `GET /api/config` returns editable config view without raw Matrix token.
  - `PUT /api/config` updates supported fields and preserves token when token field is omitted.
  - `PUT /api/config` returns `403` when the server is bound to a non-loopback address in first milestone mode.
  - Host/Origin/CORS tests reject state-changing requests from untrusted origins.
  - Config write tests prove explicit path, implicit path, first-save creation, Tauri app-config target, malformed config, and permission failures produce documented behavior.
  - TOML edit tests prove unknown keys remain and comments are preserved for existing config files where the chosen edit library supports it.
  - WebSocket sends `hello`/initial payload followed by live update messages.
  - `test -f scripts/probe-websocket.mjs` exits `0`.
  - `cargo test --all` exits `0`.

  QA scenarios:
  - Happy path HTTP: `scripts/smoke-webui.sh --scenario snapshot --evidence .omo/evidence/gui-webui-tauri/task-8-snapshot.http`; expected `200 OK` and JSON includes `session`, `missions`, `events`.
  - Failure edge token: `scripts/smoke-webui.sh --scenario config-redaction --evidence .omo/evidence/gui-webui-tauri/task-8-config-redaction.http`; expected response does not contain the fixture access token and contains only a token-present marker.
  - Remote mutation guard: `scripts/smoke-webui.sh --scenario non-loopback-config-write --evidence .omo/evidence/gui-webui-tauri/task-8-non-loopback-write.http`; expected `403` for config mutation.
  - Happy path WebSocket: `scripts/smoke-webui.sh --scenario websocket --probe scripts/probe-websocket.mjs --evidence .omo/evidence/gui-webui-tauri/task-8-websocket.jsonl`; expected first message includes snapshot and buffered events.

  Commit: YES | `feat(web): expose dashboard api`

- [x] 9. Build frontend adapters, mock data, and design-system shell

  What to do: In `ui/`, implement adapter interfaces for Web HTTP/WebSocket and future Tauri invoke/event transport. Add a mock/dev adapter backed by sanitized fixture-like data so the dashboard can be developed without real Journal files. Create shell layout, navigation, connection state, responsive grid, and shadcn/tokens wiring. Use lucide-react icons only.

  Must NOT do: Do not hardcode local absolute Journal paths. Do not use emoji icons. Do not use raw hex values outside token definitions. Do not import Tauri APIs in shared page components.

  Parallelization: Wave 3 | Blocked by: 1, 4 | Blocks: 10, 11, 12

  References:
  - `DESIGN.md`.
  - `ui/package.json`, `ui/src` from Todo 4.
  - App DTO shapes from Todo 3.
  - Official shadcn/ui component docs: https://ui.shadcn.com/docs/components
  - `.omo/knowledges/gui-readiness-2026-06-20.md` - shared frontend + adapter decision.

  Acceptance criteria:
  - `pnpm --dir ui typecheck` exits `0`.
  - `pnpm --dir ui build` exits `0`.
  - Mock adapter renders the dashboard shell without backend.
  - `rg -n "@tauri-apps|invoke\\(" ui/src/features ui/src/components ui/src/app` returns no matches outside adapter files.
  - `rg -n "🚀|✨|🔥|✅|❌" ui/src` returns no matches.

  QA scenarios:
  - Happy path browser: `pnpm --dir ui test:e2e -- --project=chromium --grep "@mock-dashboard" > .omo/evidence/gui-webui-tauri/task-9-dashboard-shell.txt`; expected test exits `0` and writes `.omo/evidence/gui-webui-tauri/task-9-dashboard-shell.png` showing shell, metrics, event feed, and mission area.
  - Failure edge: `pnpm --dir ui typecheck > .omo/evidence/gui-webui-tauri/task-9-typecheck.txt`; expected no adapter type leaks or missing DTO fields.

  Commit: YES | `feat(ui): add dashboard shell adapters`

- [x] 10. Implement the live dashboard pages and Web adapter integration

  What to do: Build the first milestone dashboard using shadcn/ui components rethemed by `DESIGN.md`: session/context header, combat metrics, health/fuel panel, warning rail, recent event feed, mission progress table, Journal source panel, Matrix/Web status, and connection states. Wire the Web adapter to HTTP snapshot/config and WebSocket updates. New clients must render backend buffered events immediately.

  Must NOT do: Do not implement replay UI. Do not add chart libraries. Do not show raw Matrix token. Do not require a real Matrix homeserver or real Elite Dangerous Journal for UI smoke tests.

  Parallelization: Wave 3 | Blocked by: 1, 7, 8, 9 | Blocks: 11, 12, 14

  References:
  - `DESIGN.md` - tokens and component rules.
  - `ui/src` adapter/shell from Todo 9.
  - Web API from Todo 8.
  - `tests/fixtures/journal_combat_bounty.log`, `tests/fixtures/journal_missions.log` - sanitized fixture behavior for dev/mock data.
  - `src/terminal.rs` - CLI status fields to preserve parity where useful.

  Acceptance criteria:
  - `pnpm --dir ui typecheck` exits `0`.
  - `pnpm --dir ui build` exits `0`.
  - Playwright opens the WebUI served by Axum and sees core dashboard regions.
  - New browser connection displays buffered previous events before new events.
  - Responsive screenshots at 375, 768, and 1280 px show no horizontal overflow.

  QA scenarios:
  - Happy path browser: `scripts/smoke-webui.sh --scenario live-dashboard --evidence .omo/evidence/gui-webui-tauri/task-10-live-dashboard.txt`; expected Playwright exits `0`, writes `.omo/evidence/gui-webui-tauri/task-10-live-dashboard.png`, and visible text includes commander/session metrics/events/missions/status.
  - Failure edge late client: `scripts/smoke-webui.sh --scenario buffered-events --evidence .omo/evidence/gui-webui-tauri/task-10-buffered-events.txt`; expected Playwright exits `0`, writes `.omo/evidence/gui-webui-tauri/task-10-buffered-events.png`, and earlier buffered events are visible in the feed.

  Commit: YES | `feat(ui): render live monitor dashboard`

- [x] 11. Implement GUI config editing

  What to do: Add config editing UI and backend handling for supported fields: Journal folder/file source where applicable, monitor thresholds/options, log levels, Web settings, Matrix enabled/status fields, Matrix room/user fields, and Matrix token replacement/clear semantics. Use forms with validation, dirty state, save/cancel, success/error messages, and explicit token masking. Backend writes local config safely and atomically where possible through the `ConfigSource`/`ConfigPath` and comment-preserving TOML edit path defined earlier.

  Must NOT do: Do not display the current raw Matrix token. Do not write secrets into evidence files. Do not silently overwrite unrelated user config comments if a structured update strategy can preserve them; if comments cannot be preserved with current TOML tooling, document that limitation before implementing the write path.

  Parallelization: Wave 3 | Blocked by: 1, 8, 9, 10 | Blocks: 14

  References:
  - `src/config.rs` - config model and load behavior.
  - `config.example.toml` - default editable schema.
  - Web API config endpoints from Todo 8.
  - `ui/src` forms and shadcn components from Todo 9/10.
  - Secret-guard skill guidance because token handling is in scope.

  Acceptance criteria:
  - Rust tests prove config update preserves Matrix token when omitted, replaces when explicitly provided, and clears only through explicit action.
  - Rust tests prove GUI save writes explicit config paths, existing implicit `./config.toml`, first-save generated `./config.toml`, and Tauri app-config path exactly as documented.
  - Rust tests prove unknown config keys survive GUI save and existing comments are preserved where supported by the selected TOML edit library.
  - Rust tests prove permission failures and malformed config return frontend-safe errors without partial writes.
  - Frontend tests or Playwright scenario prove masked token UI never renders the fixture token string.
  - Saving config updates the backend config view and is reflected after reload.
  - `cargo test --all`, `pnpm --dir ui typecheck`, and `pnpm --dir ui build` exit `0`.

  QA scenarios:
  - Happy path browser: `pnpm --dir ui test:e2e -- --project=chromium --grep "@config-edit" > .omo/evidence/gui-webui-tauri/task-11-config-edit.txt`; expected test edits a non-secret setting such as `[web].port` or monitor threshold, reloads config view, writes `.omo/evidence/gui-webui-tauri/task-11-config-edit.png`, and changed value persists.
  - Failure edge token: `pnpm --dir ui test:e2e -- --project=chromium --grep "@token-mask" > .omo/evidence/gui-webui-tauri/task-11-token-mask.txt`; expected screenshot/text dump does not contain the raw token and screenshot is saved to `.omo/evidence/gui-webui-tauri/task-11-token-mask.png`.

  Commit: YES | `feat(ui): edit monitor configuration`

- [x] 12. Add WebUI visual QA, accessibility, and performance gates

  What to do: Add Playwright tests for WebUI smoke, responsive screenshots, keyboard focus, reduced-motion behavior, empty/loading/error states, and key interactive controls. Add a production Vite build + preview/Axum served audit path. Add real-browser Lighthouse/performance checks if practical in the environment; otherwise record explicit limitation and still run Playwright visual/a11y smoke.

  Must NOT do: Do not claim visual completion from unit tests only. Do not weaken UI behavior to pass performance. Do not skip mobile/tablet/desktop breakpoints.

  Parallelization: Wave 3 | Blocked by: 1, 7, 9, 10 | Blocks: 14

  References:
  - `omo:frontend` skill Design QA requirement.
  - Frontend perfection skill real-browser audit guidance.
  - `DESIGN.md`.
  - `ui/` Playwright setup from prior tasks.

  Acceptance criteria:
  - Playwright smoke test passes against the real WebUI page.
  - Screenshots exist for 375, 768, and 1280 px.
  - Keyboard focus scenario can reach core controls.
  - `pnpm --dir ui build` exits `0`.
  - Any Lighthouse/react-scan/react-doctor checks added by setup pass or have documented environment limitation.

  QA scenarios:
  - Happy path browser: `pnpm --dir ui test:e2e -- --project=chromium > .omo/evidence/gui-webui-tauri/task-12-playwright.txt`; expected test exits `0` and screenshots are written.
  - Failure edge responsive: `pnpm --dir ui test:e2e -- --project=chromium --grep "@responsive-mobile" > .omo/evidence/gui-webui-tauri/task-12-mobile.txt`; expected Playwright captures 375 px viewport to `.omo/evidence/gui-webui-tauri/task-12-mobile.png`, with no horizontal overflow and core status visible.

  Commit: YES | `test(ui): add dashboard browser qa`

- [x] 13. Implement `ed-sentry-gui` Tauri desktop entry with shared services

  What to do: Add Tauri v2 desktop app entry named `ed-sentry-gui` using the shared `ui/` frontend. Use `ui/src-tauri/` as the Tauri crate layout. Add a Cargo workspace if needed so `ui/src-tauri` can depend on the root `ed-sentry` library crate by path. Set the Tauri product/binary name to `ed-sentry-gui`. Add Tauri frontend scripts to `ui/package.json`, including `tauri`, `tauri:dev`, and `tauri:build`, backed by the Tauri v2 CLI package rather than requiring a globally installed `cargo-tauri`. Integrate Tauri commands/events through adapter implementations, but bootstrap the same Rust application service as CLI. The desktop entry should honor `[web]` and `[matrix]` config where watch-capable, including starting WebUI server if enabled and Matrix delivery if enabled. Keep frontend pages shared; only environment adapters differ. First milestone release notes must state whether desktop artifacts are shipped by CI or only locally buildable.

  Must NOT do: Do not fork the dashboard UI. Do not make Tauri the only way to configure GUI features. Do not bypass backend token redaction. Do not route Tauri through local HTTP unless deliberately needed; prefer direct commands/events for desktop adapter while preserving optional Web server when `[web] enabled = true`.

  Parallelization: Wave 4 | Blocked by: 2, 3, 5, 6, 7, 8 | Blocks: 14, 15

  References:
  - Official Tauri v2 Vite frontend docs: https://v2.tauri.app/start/frontend/vite/
  - Official Tauri v2 calling Rust docs: https://v2.tauri.app/develop/calling-rust/
  - `ui/` adapter interface from Todo 9.
  - App service from Todo 5/6.
  - Web/Matrix startup logic from Todo 7/8 and existing `src/matrix.rs`.
  - `Cargo.toml` - binary/workspace layout.

  Acceptance criteria:
  - `cargo test --all` exits `0`.
  - `test -f ui/src-tauri/tauri.conf.json` exits `0`.
  - `pnpm --dir ui tauri build` exits `0` in the supported desktop-capable environment, or records a concrete missing-system-dependency blocker while `pnpm --dir ui build` and Rust service tests still pass.
  - `ed-sentry-gui` uses shared frontend source, not a copied UI.
  - Desktop startup tests or smoke checks prove config-enabled Web/Matrix startup paths are called through shared service seams.
  - Tauri adapter files are the only frontend files importing `@tauri-apps/api` or calling `invoke`.
  - Docs/release notes explicitly state whether CI publishes `ed-sentry-gui` artifacts in milestone one.

  QA scenarios:
  - Happy path desktop/build: `pnpm --dir ui tauri build > .omo/evidence/gui-webui-tauri/task-13-tauri-build.txt`; expected command exits `0`, or the evidence records the concrete missing Tauri system dependency.
  - Failure edge service parity: `cargo test --lib app::tests::desktop_bootstrap_honors_web_and_matrix_config > .omo/evidence/gui-webui-tauri/task-13-tauri-web-start.txt`; expected Web startup event/warning behavior matches CLI service.

  Commit: YES | `feat(gui): add tauri desktop entry`

- [x] 14. Update docs, example config, release packaging, and secret-scan commands for WebUI/Tauri

  What to do: Update README and `config.example.toml` with `[web]` configuration, WebUI startup behavior, local-first security warning, config editing behavior, loopback-only config mutation, Web/Tauri shared frontend model, `ed-sentry-gui` desktop entry, verification commands, and scope boundaries. Update release workflow/package scripts for the phase-one non-embedded WebUI asset strategy: build `ui/dist`, copy it to a `webui/` directory beside `ed-sentry` in release archives, and verify packaged `/` serving. Document desktop release status explicitly: either CI publishes `ed-sentry-gui` artifacts in milestone one, or CI docs state the exact local `pnpm --dir ui tauri build` path and the tracked blocker. Add concrete secret/privacy scan commands.

  Must NOT do: Do not document a `--webui` flag. Do not suggest exposing WebUI publicly without warning. Do not include real tokens or raw Journal examples. Do not claim historical database/replay GUI support.

  Parallelization: Wave 4 | Blocked by: 10, 11, 12, 13 | Blocks: 15

  References:
  - `README.md` - current CLI/config/release docs.
  - `config.example.toml` - safe committed config template.
  - `.omo/knowledges/gui-readiness-2026-06-20.md` - decisions to document.
  - `.github/workflows/ci.yml`, `.github/workflows/release.yml` - CI/release reality.

  Acceptance criteria:
  - README documents `[web] enabled = true` startup and no `--webui`.
  - README documents WebUI replay exclusion and local-first security model.
  - README documents that config mutation is loopback-only in the first milestone.
  - README/release docs document `ED_SENTRY_WEBUI_DIST`, sibling `webui/`, and repo-local `ui/dist` lookup order.
  - `config.example.toml` includes safe `[web]` defaults.
  - Release workflow or package scripts build frontend assets and include `webui/` beside packaged `ed-sentry`.
  - Release docs state whether `ed-sentry-gui` is published by CI or only locally buildable in the first milestone.
  - `rg -n -- '--webui|access_token = \"[^\"]{8,}\"|historical database|GUI replay' README.md config.example.toml` returns no false user-facing claims.
  - Secret/privacy scan commands below exit with expected status and produce evidence.
  - `cargo test --all` and frontend build gates still pass after docs/config changes.

  QA scenarios:
  - Happy path docs grep: `rg -n "\\[web\\]|ed-sentry-gui|127.0.0.1|WebUI" README.md config.example.toml > .omo/evidence/gui-webui-tauri/task-14-docs.txt`; expected output shows documented config and startup behavior.
  - Packaged asset docs: `rg -n "ED_SENTRY_WEBUI_DIST|webui/|ui/dist|tauri build|ed-sentry-gui" README.md .github/workflows/release.yml scripts > .omo/evidence/gui-webui-tauri/task-14-packaging-docs.txt`; expected output shows asset packaging and desktop build/release status.
  - Failure edge secrets/scope: `rg -n --hidden --glob '!target/**' --glob '!ui/node_modules/**' --glob '!ui/dist/**' --glob '!dist/**' --glob '!Cargo.lock' 'access_token\\s*=\\s*\"[^\"<][^\"]{8,}\"|Matrix access token:|Journal\\.[0-9].*\\.log|Elite Dangerous|BEGIN (RSA|OPENSSH|PRIVATE) KEY' README.md config.example.toml src tests ui .omo/evidence/gui-webui-tauri > .omo/evidence/gui-webui-tauri/task-14-docs-guard.txt`; expected command exits `1` with no matches. If expected anti-pattern prose must remain, narrow the command and document why.

  Commit: YES | `docs(gui): document web and desktop ui`

- [x] 15. Run release-readiness hardening and full verification

  What to do: Add and run `scripts/verify-gui-webui-tauri.sh`, the full Rust + frontend + WebUI + Tauri verification set. Fix failures within scope. Confirm no product code exposes raw Matrix tokens, no raw Journal private content entered fixtures/evidence, no duplicate monitor loop exists, replay remains terminal-only, and WebUI failure remains best-effort. Capture final evidence and update `.omo/knowledges/gui-webui-tauri.md` with implementation facts.

  Must NOT do: Do not delete or weaken tests. Do not skip visual/browser QA. Do not commit generated secrets, `config.toml`, `node_modules`, or raw Journal files. Do not declare complete from green unit tests alone.

  Parallelization: Wave 5 | Blocked by: 5, 6, 8, 13, 14 | Blocks: final verification

  References:
  - All changed source/docs/tests.
  - `README.md` verification commands.
  - `.gitignore` and secret-guard guidance.
  - Final plan success criteria below.

  Acceptance criteria:
  - `test -x scripts/verify-gui-webui-tauri.sh` exits `0`.
  - `cargo fmt --check` exits `0`.
  - `cargo test --all` exits `0`.
  - `cargo clippy --all-targets --all-features -- -D warnings` exits `0`.
  - `pnpm --dir ui install --frozen-lockfile`, `pnpm --dir ui typecheck`, and `pnpm --dir ui build` exit `0`.
  - Playwright WebUI smoke/visual QA exits `0`.
  - `scripts/smoke-webui.sh --scenario packaged-assets --evidence .omo/evidence/gui-webui-tauri/task-15-packaged-assets.http` verifies the packaged artifact can serve `/`.
  - `pnpm --dir ui tauri build` exits `0` or records a concrete environment blocker with non-Tauri GUI/WebUI verification still passing.
  - Secret scan finds no token leaks.
  - New knowledge file exists at `.omo/knowledges/gui-webui-tauri.md`.

  QA scenarios:
  - Happy path full gate: `scripts/verify-gui-webui-tauri.sh > .omo/evidence/gui-webui-tauri/task-15-full-gate.txt`; expected all required commands exit `0`, except an explicitly recorded Tauri system-dependency blocker is allowed only when Rust/frontend/WebUI browser/package checks still pass.
  - Failure edge replay isolation: `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line > .omo/evidence/gui-webui-tauri/task-15-replay-isolation.txt`; expected replay output contains terminal summary and no WebUI/Matrix startup.
  - Browser surface: `scripts/smoke-webui.sh --scenario production-dashboard --evidence .omo/evidence/gui-webui-tauri/task-15-production-dashboard.txt`; expected Playwright saves `.omo/evidence/gui-webui-tauri/task-15-production-dashboard.png` and dashboard renders live/snapshot data.

  Commit: YES | `chore(gui): verify web and desktop ui`

## Final verification wave
> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.
- [x] F1. Plan compliance audit
  - Verify every Must Have is represented in code/docs/tests or explicitly deferred in the plan.
  - Verify every Must NOT Have remains absent.
  - Verify all task evidence files exist under `.omo/evidence/gui-webui-tauri/`.
  - Command evidence: `rg -n "\\[web\\]|ed-sentry-gui|WebSocket|shadcn|DESIGN.md|Matrix token|replay" .omo/evidence/gui-webui-tauri README.md config.example.toml src ui`.

- [x] F2. Code quality review
  - Review app-service boundaries: CLI, WebUI, and Tauri must share runtime logic.
  - Review DTOs for token/raw-Journal exposure.
  - Review frontend for design-system token use, adapter separation, and generic shadcn visual drift.
  - Review that no duplicate watch loop exists.

- [x] F3. Real manual QA
  - Run CLI watch/WebUI server against sanitized temp Journal and capture HTTP/WebSocket/browser evidence.
  - Run replay fixture and confirm no WebUI/Matrix startup.
  - Run desktop GUI smoke/build evidence for `ed-sentry-gui`.
  - Capture browser screenshots at 375, 768, and 1280 px.

- [x] F4. Scope fidelity
  - Confirm no GUI replay, no historical database, no auth/public remote mode, no chart library, no Matrix command handling, no automation.
  - Confirm no `--webui` flag was introduced.
  - Confirm local-first warning behavior for non-localhost bind.
  - Confirm `config.toml`, raw Journals, `node_modules`, and secret values are untracked/uncommitted.

## Commit strategy
- Use atomic Conventional Commits.
- Commit groups should follow plan boundaries: docs/design, config, app DTO/service, web backend, frontend scaffold, dashboard UI, config editing, Tauri, docs, verification.
- Do not auto-commit unless the user explicitly asks for commits. If committing is requested later, stage only files belonging to the relevant task and include `Plan: .omo/plans/gui-webui-tauri.md` in final implementation commit footers.
- Keep generated or local-only artifacts out of commits: `config.toml`, raw Journal files, `node_modules`, temporary evidence unless the project intentionally tracks a small sanitized evidence file.
- If package lock files are created (`ui/pnpm-lock.yaml`), commit them with the frontend scaffold task.

## Success criteria
- The existing CLI behavior still works: default watch mode remains no-subcommand, `--replay` remains terminal-only, and existing sanitized tests pass.
- `[web] enabled = true` in config starts a local-first WebUI server in watch-capable CLI and desktop GUI runtimes; no separate `--webui` flag is required.
- WebUI startup failure is warning-only and does not stop core monitoring.
- The WebUI backend owns Journal reading/watch/config access and exposes sanitized HTTP/WebSocket data.
- A backend recent-event buffer lets newly opened clients see earlier current-process events.
- GUI config editing works for supported fields without leaking raw Matrix tokens.
- Shared React/Vite/shadcn frontend under `ui/` renders the live operational dashboard through adapters and is reused by Tauri.
- `ed-sentry-gui` is a separate desktop entry that bootstraps the same Rust application services and honors config-enabled Web/Matrix behavior.
- No GUI replay, historical database, auth/public remote mode, chart library, or automation was added.
- Required Rust, frontend, WebUI browser, and desktop/Tauri verification commands pass or record a concrete environment limitation for desktop execution only.
