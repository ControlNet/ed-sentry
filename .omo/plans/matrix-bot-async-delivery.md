# Matrix Bot Async Delivery

## TL;DR
> **Summary**: Refactor `ed-sentry` so one async executable can keep the current CLI display while also delivering the same watch-mode business notifications to one unencrypted Matrix room via `matrix-sdk`. Replay remains terminal-only, Matrix failures are best-effort warnings, and live progress is mirrored by editing one Matrix status message.
> **Deliverables**:
> - `EventMonitor` becomes a notification producer instead of owning synchronous delivery.
> - Async `main`/watch orchestration with terminal delivery plus optional Matrix delivery.
> - `config.toml` auto-load, Matrix config parsing, `.gitignore` protection, and updated docs/examples.
> - Matrix SDK delivery with access-token restore, user mentions for level `>= 2`, and editable status message.
> - Automated fake-Matrix tests proving routing, replay isolation, status editing, and best-effort failure behavior.
> **Effort**: XL
> **Parallel**: YES - 4 waves with serial subwaves where producer/delivery refactors require ordering
> **Critical Path**: Task 1 → Task 2 → Task 5 → Task 7 → Task 9 → Task 12

## Context
### Original Request
Implement the next sending-side feature as a Matrix bot mode, but do it carefully because this is a large refactor. The bot should run with the existing CLI executable in watch mode, mirror CLI business notifications to Matrix, mention a configured user for critical events, keep editing a single Matrix status message, use `matrix-sdk`, convert the main flow to async, and keep replay mode Matrix-free.

### Interview Summary
- Four presentation modes are envisioned long-term: CLI, Matrix bot, Web, Tauri desktop. This plan covers only CLI + Matrix bot.
- CLI is already implemented and remains the local primary display.
- Matrix is an additional sink, not a replacement for terminal output.
- Output routing is now three-tier: `0 = no send`, `1 = CLI + Matrix`, `2+ = CLI + Matrix + Matrix mention`.
- Matrix delivery uses `config.toml` with direct `access_token = "<token>"`; no `access_token_env`.
- If `--config` is absent, the app auto-loads `./config.toml` if present, otherwise uses defaults. Explicit `--config` remains strict.
- `config.toml` is ignored by git; `config.example.toml` is committed and safe.
- Matrix E2EE is unsupported and documented only for MVP; do not implement runtime encrypted-room detection.
- Use `matrix-sdk`, async main orchestration, and a full `EventMonitor` producer refactor rather than a collecting-notifier bridge.
- Matrix `device_id` is a fixed default: `EDAFKDASHBOARD`.

### Metis Review (gaps addressed)
- Repo path is `/home/ubuntu/GitRepos/ed-sentry`, while package/binary is `ed-sentry`; this is not a blocker, but references must use exact current paths.
- Replay must be Matrix-free by construction: no Matrix client construction, session restore, `sync_once`, room lookup, send, edit, or status message in `--replay`.
- Matrix slowness/failure must not kill or block terminal monitoring beyond one awaited best-effort attempt; warnings must be line-safe and token-redacted.
- Existing `Notification::new` derives `mention` from `level >= 3`; this plan changes it to `level >= 2` and updates tests/docs/defaults accordingly.
- Tests must use fake Matrix delivery; normal verification must not require real homeserver credentials.
- Terminal rendering must stay deterministic and protected by existing tests.

## Work Objectives
### Core Objective
Make watch mode fan out notifications to both terminal and Matrix while preserving existing CLI behavior and making Matrix optional, best-effort, and never active in replay.

### Deliverables
- Async runtime entrypoint in `src/main.rs` using `tokio`.
- `EventMonitor` producer API in `src/monitor.rs` returning notifications instead of dispatching to `Notifier`.
- Delivery layer for terminal + Matrix fanout.
- `src/matrix.rs` Matrix SDK integration and fakeable Matrix sender/status abstractions.
- `MatrixConfig` parsing and runtime validation in `src/config.rs`.
- `.gitignore` update for `/config.toml`.
- `config.example.toml` and README updates reflecting new config and log-level semantics.
- Regression tests for producer parity, config auto-load, routing, replay isolation, Matrix mention, status edit, and best-effort errors.

### Definition of Done (verifiable conditions with commands)
- `cargo fmt --check` exits `0`.
- `cargo test --all` exits `0` without real Matrix credentials.
- `cargo clippy --all-targets --all-features -- -D warnings` exits `0`.
- `cargo run -- --help` exits `0` and still shows no subcommands.
- `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` exits `0` and performs no Matrix initialization in test-covered code paths.
- `cargo run -- --replay --no-status-line` exits non-zero with `Error: replay requires --set-file <file>`.
- Test artifacts prove `config.toml` auto-loads when no `--config` is supplied and is ignored by git.

### Must Have
- `level == 0`: no CLI delivery and no Matrix delivery, while state still updates where current behavior updates state.
- `level == 1`: terminal delivery plus Matrix message, no Matrix mention.
- `level >= 2`: terminal delivery plus Matrix message with configured Matrix mention metadata when `mention_user_id` is configured.
- Matrix text body is built from `Notification.remote_text` plus optional emoji and optional mention prefix; never from raw Journal payload.
- Matrix status uses one original status event ID; subsequent status updates edit that original event, not the previous edit event.
- Matrix initialization failures and send/edit failures print warnings and continue CLI monitoring.
- Matrix connect/session restore/`sync_once` operations are wrapped in `tokio::time::timeout(Duration::from_secs(10), ...)`; Matrix send/edit/status operations are wrapped in `tokio::time::timeout(Duration::from_secs(5), ...)`. Timeout means warning + Matrix disabled for startup or warning + skipped message/status for runtime delivery.
- Replay uses terminal-only delivery even if `config.toml` contains `[matrix] enabled = true`.

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)
- Do not implement Matrix command handling, sync-loop message handling, daemon/service mode, multi-room routing, Discord/Telegram, Web/Tauri, durable retry queue, HTML/rich formatting, token env vars, secret manager integration, E2EE support, or encrypted-room detection.
- Do not log or debug-print `access_token`.
- Do not add `config.local.toml` or document it as a supported standard path.
- Do not rewrite Journal parsing, state modeling, terminal rendering, or fixtures except where tests require API adaptation.
- Do not require a real Matrix homeserver/token for CI or normal tests.

## Verification Strategy
> ZERO HUMAN INTERVENTION - all verification is agent-executed.
- Test decision: TDD + tests-after, using existing Rust test suite plus new fake Matrix tests.
- QA policy: Every task has agent-executed scenarios.
- Evidence: `.omo/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy
### Parallel Execution Waves
> Target: 5-8 tasks per wave. <3 per wave (except final) = under-splitting.
> Extract shared dependencies as Wave-1 tasks for max parallelism.

Wave 1: Tasks 1-4 foundation and behavior locks (`quick`, `unspecified-high`, `writing`)
Wave 2: Tasks 5-8 producer refactor, async orchestration, delivery abstractions (`deep`, `unspecified-high`); execute as ordered subwaves 5 → 7 → 8, with Task 6 parallel after Task 1
Wave 3: Tasks 9-11 Matrix SDK delivery, watch integration, docs (`deep`, `writing`); execute 9 → 10 → 11
Wave 4: Task 12 end-to-end hardening and compatibility (`unspecified-high`)

### Dependency Matrix (full, all tasks)
- Task 1 blocks Tasks 6, 9, 10, 11.
- Task 2 blocks Tasks 5, 7, 8, 10, 12.
- Task 3 blocks Tasks 9, 10, 11.
- Task 4 blocks Tasks 11, 12.
- Task 5 blocks Tasks 7, 8, 10, 12.
- Task 6 blocks Tasks 9, 10, 11.
- Task 7 blocks Tasks 8, 10, 12.
- Task 8 blocks Task 10.
- Task 9 blocks Tasks 10, 12.
- Task 10 blocks Task 12.
- Task 11 blocks Task 12 and must run after Task 10 because it documents verified integration behavior.
- Task 12 blocks final verification only.

### Agent Dispatch Summary (wave → task count → categories)
- Wave 1 → 4 tasks → `quick` x1, `unspecified-high` x2, `writing` x1
- Wave 2 → 4 tasks → `deep` x2, `unspecified-high` x2; Task 6 can run in parallel with the serial 5→7→8 path
- Wave 3 → 3 tasks → `deep` x2, `writing` x1; documentation task follows integration to avoid stale docs
- Wave 4 → 1 task → `unspecified-high` x1

## TODOs
> Implementation + Test = ONE task. Never separate.
> EVERY task MUST have: Agent Profile + Parallelization + QA Scenarios.

- [x] 1. Matrix config model, auto-load, and gitignore protection

  **What to do**: In `src/config.rs`, add `MatrixConfig` and carry it through `AppConfig` and `RuntimeConfig` as `matrix: Option<MatrixConfig>`. Parse `[matrix]` manually using the existing `toml::Value` style, but do **not** perform runtime required-field validation in this task. Defaults: missing `[matrix]` = `None`; `[matrix] enabled = false` = parsed disabled config or `None` according to the cleanest implementation; `[matrix] enabled = true` stores whatever typed fields are present for Task 6 to validate; `mention_user_id` optional; `status_update_interval_seconds` defaults to `60`; `device_id` is not user-configurable and must resolve to fixed `EDAFKDASHBOARD` in Matrix runtime setup. Change `AppConfig::load_optional(None)` to check `Path::new("config.toml")`: if it exists, load it; if it does not, return defaults. Explicit `--config <path>` remains strict. Add `/config.toml` to `.gitignore`. Add config unit/integration tests for defaults, implicit auto-load, explicit strict missing config, malformed implicit config error, Matrix enabled/disabled parsing, wrong-typed Matrix keys warning, and token redaction.
  **Must NOT do**: Do not add `access_token_env`, `config.local.toml`, Matrix CLI flags, or any real token fixture.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: config behavior is cross-cutting and has security implications.
  - Skills: [`secret-guard`] - Token handling and gitignore behavior matter.
  - Omitted: [`playwright`] - No browser/UI work.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: Tasks 6, 9, 10, 11 | Blocked By: none

  **References** (executor has NO interview context - be exhaustive):
  - Pattern: `src/config.rs:7` - `AppConfig` currently contains `journal`, `monitor`, `log_levels` only.
  - Pattern: `src/config.rs:201` - `AppConfig::from_toml_str`, `load_from_path`, and `load_optional` are the config loading seam.
  - Pattern: `src/config.rs:256` - manual `toml::Value` parsing style should be followed for `[matrix]`.
  - Pattern: `src/main.rs:77` - `build_runtime_command` calls `AppConfig::load_optional(cli.config.as_deref())`.
  - Pattern: `.gitignore:1` - add `/config.toml` near other root/local/generated ignores.
  - Test: `tests/cli_config.rs` - integration tests already exercise config CLI behavior.

  **Acceptance Criteria** (agent-executable only):
  - [ ] `cargo test --test cli_config cli_config_implicit_config_toml_loads_when_config_flag_absent` passes.
  - [ ] `cargo test --test cli_config cli_config_explicit_missing_config_still_errors` passes.
  - [ ] `cargo test --lib config::tests::config_matrix_enabled_preserves_present_fields_for_runtime_validation` passes.
  - [ ] `git check-ignore config.toml` exits `0` and prints `config.toml`.
  - [ ] `cargo test --all` passes with no real Matrix token.

  **QA Scenarios** (MANDATORY - task incomplete without these):
  ```
  Scenario: Implicit config.toml is optional
    Tool: Bash
    Steps: cargo test --test cli_config cli_config_implicit_config_toml_absent_uses_defaults
    Expected: Test exits 0 and asserts default config is used when no --config and no config.toml exist in test working directory.
    Evidence: .omo/evidence/task-1-config-autoload.txt

  Scenario: Explicit missing config remains strict
    Tool: Bash
    Steps: cargo test --test cli_config cli_config_explicit_missing_config_still_errors
    Expected: Test exits 0 and asserts binary exits non-zero with failed-to-read-config error.
    Evidence: .omo/evidence/task-1-config-explicit-error.txt
  ```

  **Commit**: CONDITIONAL | Message: `Support local Matrix config loading` | Files: [`src/config.rs`, `src/main.rs`, `.gitignore`, `tests/cli_config.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 2. Notification routing semantics and monitor behavior locks

  **What to do**: Update `Notification::new` in `src/notifier.rs` so `mention` is `level >= 2`. Update log-level comments/docs/defaults so the public meaning is `0 = no send`, `1 = notify`, `2+ = notify and Matrix mention`. Convert current default `3` values in `LogLevelConfig::default()` and `config.example.toml` to `2` where the intent is critical mention. Add/adjust tests that prove `level 0` is suppressed, `level 1` is non-mention, and `level 2`/`3` are mention-capable. Before producer refactor, add or update tests in `tests/monitor_events.rs`, `tests/warnings.rs`, and `tests/notifier.rs` to lock current notification texts, event types, levels, and state updates so Task 5 can refactor safely.
  **Must NOT do**: Do not change terminal line formatting or business notification text except where docs/comments describe levels.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: public routing semantics affect many tests.
  - Skills: [] - No specialized external skill needed.
  - Omitted: [`secret-guard`] - No secrets should be touched.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: Tasks 5, 7, 8, 10, 12 | Blocked By: none

  **References**:
  - Pattern: `src/notifier.rs:12` - `Notification` has `level`, `remote_text`, and `mention`.
  - Pattern: `src/notifier.rs:24` - `Notification::new` currently derives mention from level.
  - Pattern: `src/notifier.rs:99` - dispatcher currently filters `level == 0`.
  - Pattern: `src/config.rs:166` - default log levels currently include `3` for critical events.
  - Pattern: `config.example.toml:29` - current log-level comments still describe old Phase 1 remote-capable levels.
  - Test: `tests/monitor_events.rs` - broad notification behavior tests.
  - Test: `tests/warnings.rs` - warning scheduler notifications.
  - Test: `tests/notifier.rs` - public notifier API driver.

  **Acceptance Criteria**:
  - [ ] `cargo test --lib notifier::tests::notifier_level_routing_ignores_zero_and_marks_mentions_at_two` passes.
  - [ ] `cargo test --test monitor_events monitor_events_level_zero_updates_state_without_delivery` passes.
  - [ ] `cargo test --test warnings` passes.
  - [ ] `cargo test --all` passes after default/config-example level updates.

  **QA Scenarios**:
  ```
  Scenario: Level 2 is mention-capable
    Tool: Bash
    Steps: cargo test --lib notifier::tests::notifier_level_two_is_mention_capable
    Expected: Test exits 0 and asserts level 1 mention=false, level 2 mention=true, level 3 mention=true.
    Evidence: .omo/evidence/task-2-routing-mentions.txt

  Scenario: Level zero still updates state without delivery
    Tool: Bash
    Steps: cargo test --test monitor_events monitor_events_level_zero_updates_state_without_delivery
    Expected: Test exits 0 and asserts monitor state changes while emitted notifications exclude level-zero delivery.
    Evidence: .omo/evidence/task-2-level-zero-state.txt
  ```

  **Commit**: CONDITIONAL | Message: `Update notification routing levels` | Files: [`src/notifier.rs`, `src/config.rs`, `config.example.toml`, `tests/notifier.rs`, `tests/monitor_events.rs`, `tests/warnings.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 3. Dependency and async runtime foundation

  **What to do**: Add dependencies to `Cargo.toml`: `matrix-sdk = "0.18.0"`, `tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "signal"] }`, and `async-trait = "0.1"` if the chosen delivery traits require async trait objects. Convert `src/main.rs::main` to `#[tokio::main] async fn main() -> ExitCode`, make `run_command`, `run_watch`, and `run_replay` async signatures, and replace `thread::sleep(live_poll_interval(config))` with `tokio::time::sleep(live_poll_interval(config)).await`. Keep behavior terminal-only in this task; do not add Matrix construction yet. Remove `use std::thread` if no longer needed. Ensure existing binary tests still pass.
  **Must NOT do**: Do not introduce Matrix network calls, delivery fanout, or command handling in this task.

  **Recommended Agent Profile**:
  - Category: `quick` - Reason: controlled runtime plumbing with existing behavior retained.
  - Skills: [] - No special skill needed.
  - Omitted: [`debugging`] - Not a runtime bug investigation unless tests fail unexpectedly.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: Tasks 9, 10, 11 | Blocked By: none

  **References**:
  - Pattern: `Cargo.toml:6` - current dependencies are minimal and do not include async/Matrix crates.
  - Pattern: `src/main.rs:66` - current `main` is synchronous.
  - Pattern: `src/main.rs:121` - `run_command` dispatches watch/replay synchronously.
  - Pattern: `src/main.rs:144` - `run_watch` is current sync watch loop.
  - Pattern: `src/main.rs:210` - current watch loop sleeps via `thread::sleep`.
  - Test: `tests/cli_config.rs` - binary watch/replay behavior must remain observable.

  **Acceptance Criteria**:
  - [ ] `cargo test --test cli_config cli_config_watch_tails_until_stopped` passes.
  - [ ] `cargo test --test replay` passes.
  - [ ] `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` exits `0`.
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings` has no unused sync imports.

  **QA Scenarios**:
  ```
  Scenario: Async main preserves replay behavior
    Tool: Bash
    Steps: cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
    Expected: Exit 0 and stdout still contains replay summary fragments such as Total Stats.
    Evidence: .omo/evidence/task-3-async-replay.txt

  Scenario: Async watch remains stoppable in integration test
    Tool: Bash
    Steps: cargo test --test cli_config cli_config_watch_tails_until_stopped
    Expected: Test exits 0 without hanging; watch mode still tails and can be terminated by test harness.
    Evidence: .omo/evidence/task-3-async-watch.txt
  ```

  **Commit**: CONDITIONAL | Message: `Run CLI orchestration on Tokio` | Files: [`Cargo.toml`, `Cargo.lock`, `src/main.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 4. Documentation and example config contract update

  **What to do**: Update `README.md` and `config.example.toml` to document the new config behavior, Matrix MVP, and boundaries. `config.example.toml` must include `[matrix] enabled = false`, `homeserver`, `user_id`, `room_id`, `access_token = "<token>"`, optional `mention_user_id`, and `status_update_interval_seconds = 60`; it must explicitly say Matrix requires an unencrypted room and E2EE is unsupported. README must explain `cp config.example.toml config.toml`, that `config.toml` is local/gitignored, direct token storage, default auto-load, explicit `--config`, replay no-Matrix behavior, and level semantics `0/1/2+`. Remove or replace startup/documentation language that implies Discord webhook behavior.
  **Must NOT do**: Do not document `config.local.toml`, `access_token_env`, E2EE setup, command handling, or real tokens.

  **Recommended Agent Profile**:
  - Category: `writing` - Reason: docs must be precise and user-facing.
  - Skills: [`secret-guard`] - Avoid leaking token-like examples.
  - Omitted: [`github-cli`] - No remote GitHub inspection needed.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: Tasks 11, 12 | Blocked By: none

  **References**:
  - Pattern: `README.md:92` - current configuration section.
  - Pattern: `README.md:122` - old log-level semantics.
  - Pattern: `README.md:148` - Phase 2 Matrix roadmap currently says Matrix deferred.
  - Pattern: `config.example.toml:1` - Phase 1 comment must become Matrix-aware.
  - Pattern: `config.example.toml:29` - log-level comments need new semantics.
  - Pattern: `src/main.rs:384` - startup prints old Discord webhook text.

  **Acceptance Criteria**:
  - [ ] `python - <<'PY'\nfrom pathlib import Path\ntext=Path('config.example.toml').read_text()\nassert 'access_token = "<token>"' in text\nassert 'access_token_env' not in text\nassert 'E2EE' in text or 'end-to-end' in text.lower()\nPY` exits `0`.
  - [ ] `python - <<'PY'\nfrom pathlib import Path\ntext=Path('README.md').read_text()\nassert 'config.toml' in text\nassert 'replay' in text.lower() and 'Matrix' in text\nassert 'access_token_env' not in text\nPY` exits `0`.
  - [ ] `cargo test --all` passes after doctext/example updates.

  **QA Scenarios**:
  ```
  Scenario: Example config is committed-safe
    Tool: Bash
    Steps: python - <<'PY'
from pathlib import Path
text=Path('config.example.toml').read_text()
assert 'access_token = "<token>"' in text
assert 'syt_' not in text
assert 'access_token_env' not in text
PY
    Expected: Exit 0; example uses only placeholder token and no env-token field.
    Evidence: .omo/evidence/task-4-example-config-safe.txt

  Scenario: README documents replay no-Matrix boundary
    Tool: Bash
    Steps: python - <<'PY'
from pathlib import Path
text=Path('README.md').read_text().lower()
assert 'replay' in text and 'matrix' in text and ('does not send' in text or 'never sends' in text)
PY
    Expected: Exit 0; README clearly states replay does not send Matrix.
    Evidence: .omo/evidence/task-4-readme-replay-boundary.txt
  ```

  **Commit**: CONDITIONAL | Message: `Document Matrix config contract` | Files: [`README.md`, `config.example.toml`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 5. Refactor EventMonitor into a notification producer

  **What to do**: In `src/monitor.rs`, remove the `N: Notifier` generic and the embedded `NotificationDispatcher<N>` from `EventMonitor`. Change constructors to `EventMonitor::new(monitor_config, log_levels)` and `EventMonitor::from_runtime_config(config)`. Change methods: `process_event(&mut self, event) -> Vec<Notification>`, `check_warnings_at(&mut self, now, preload) -> Vec<Notification>`, `start_monitor(...) -> Notification`, and `finish(...) -> Vec<Notification>` or an equivalent non-dispatching return shape. Preserve state mutation ordering exactly: apply state before generating event notifications and warning resets as current code does. Update tests to assert returned notifications directly instead of reading `FakeNotifier`. Keep `src/notifier.rs::Notification` as the platform-independent model; remove or deprecate `Notifier`/`NotificationDispatcher` only if all public tests are migrated, otherwise leave minimal compatibility only where still tested.
  **Must NOT do**: Do not introduce async into monitor business logic; do not change notification text, event_type, remote_text, or terminal_text except due to level semantics from Task 2.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: central architectural refactor with many tests.
  - Skills: [] - Core Rust refactor only.
  - Omitted: [`matrix-sdk`] - Matrix delivery is not implemented in this task.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: Tasks 7, 8, 10, 12 | Blocked By: Task 2

  **References**:
  - Pattern: `src/monitor.rs:16` - `EventMonitor<N>` currently owns state and dispatcher.
  - Pattern: `src/monitor.rs:95` - constructor is generic over `Notifier`.
  - Pattern: `src/monitor.rs:124` - `process_event` currently dispatches inline.
  - Pattern: `src/monitor.rs:143` - `check_warnings_at` currently dispatches inline.
  - Pattern: `src/monitor.rs:188` - `finish` dispatches summary/stopped notifications.
  - Pattern: `src/monitor.rs:201` - `start_monitor` dispatches start notification.
  - Test: `tests/monitor_events.rs` and `tests/warnings.rs` - migrate from fake notifier extraction to returned notifications.
  - Knowledge: `.omo/knowledges/task-5-notifier-dispatcher.md` - old Phase 1 notifier abstraction rationale.

  **Acceptance Criteria**:
  - [ ] `cargo test --test monitor_events` passes with direct returned-notification assertions.
  - [ ] `cargo test --test warnings` passes with direct returned-notification assertions.
  - [ ] `cargo test --test notifier` either passes unchanged via compatibility helpers or is intentionally updated to the new notification-model boundary.
  - [ ] `rg -n "EventMonitor<|NotificationDispatcher<|from_runtime_config\(TerminalNotifier" src tests` returns no obsolete monitor-owned-dispatch patterns, except compatibility tests explicitly named as such.

  **QA Scenarios**:
  ```
  Scenario: Producer emits previous monitor notifications
    Tool: Bash
    Steps: cargo test --test monitor_events
    Expected: Exit 0; tests assert same business notifications are returned from EventMonitor methods.
    Evidence: .omo/evidence/task-5-monitor-producer.txt

  Scenario: Warning scheduler still emits notifications
    Tool: Bash
    Steps: cargo test --test warnings
    Expected: Exit 0; no-kill and low-rate warnings are returned without monitor-owned dispatch.
    Evidence: .omo/evidence/task-5-warning-producer.txt
  ```

  **Commit**: CONDITIONAL | Message: `Make monitor produce notifications` | Files: [`src/monitor.rs`, `src/notifier.rs`, `tests/monitor_events.rs`, `tests/warnings.rs`, `tests/notifier.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 6. Matrix config validation and token-safe runtime setup helpers

  **What to do**: Add helper methods in `src/config.rs` or new `src/matrix.rs` config submodule to turn `Option<MatrixConfig>` into a watch-mode `MatrixRuntimeConfig` only when Matrix is enabled and required fields are present. Fields: `homeserver`, `user_id`, `room_id`, `access_token`, `mention_user_id: Option<String>`, `status_update_interval_seconds`, fixed `device_id()` returning `EDAFKDASHBOARD`. Missing required fields when `enabled=true` should produce one line-safe warning reason and disable Matrix for the run; missing `[matrix]` or `enabled=false` should be silent. Implement `Debug` manually or avoid deriving it for any struct that contains `access_token`, so debug output redacts token as `<redacted>`. Add tests for redaction and validation.
  **Must NOT do**: Do not contact Matrix, instantiate SDK client, or add encrypted-room detection.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: validation and token redaction are security-sensitive.
  - Skills: [`secret-guard`] - Token handling must be audited.
  - Omitted: [`playwright`] - No UI/browser work.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: Tasks 9, 10, 11 | Blocked By: Task 1

  **References**:
  - Pattern: `src/config.rs:93` - `LoadedConfig` returns warnings.
  - Pattern: `src/config.rs:99` - `ConfigError` display should not include token values.
  - Pattern: `src/main.rs:121` - config warnings are printed as `Warning: ...`.
  - User decision: `access_token` is direct config field; no env indirection.
  - User decision: `device_id` fixed default `EDAFKDASHBOARD`.

  **Acceptance Criteria**:
  - [ ] `cargo test --lib config::tests::matrix_runtime_config_redacts_access_token` passes.
  - [ ] `cargo test --lib config::tests::matrix_enabled_missing_required_field_disables_with_warning` passes.
  - [ ] `cargo test --all` output does not include fixture access token string from tests.

  **QA Scenarios**:
  ```
  Scenario: Token is redacted in debug and warnings
    Tool: Bash
    Steps: cargo test --lib matrix_config_redacts_access_token -- --nocapture
    Expected: Exit 0; captured output/assertions show no raw fixture token appears.
    Evidence: .omo/evidence/task-6-token-redaction.txt

  Scenario: Partial Matrix config disables Matrix with warning
    Tool: Bash
    Steps: cargo test --lib matrix_enabled_missing_required_field_disables_with_warning
    Expected: Exit 0; enabled=true without room_id/access_token yields warning and no runtime Matrix config.
    Evidence: .omo/evidence/task-6-partial-config-warning.txt
  ```

  **Commit**: CONDITIONAL | Message: `Validate Matrix runtime config safely` | Files: [`src/config.rs`, `src/matrix.rs`, `src/lib.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 7. Async terminal delivery and fanout delivery layer

  **What to do**: Add a concrete delivery layer that can send producer notifications outside `EventMonitor`. Implement `src/delivery.rs` with these fixed interfaces: `pub struct DeliveryWarning { pub message: String }`; `#[async_trait] pub trait RemoteDelivery: Send { async fn send(&mut self, notification: &Notification) -> anyhow::Result<()>; async fn publish_status(&mut self, status: &str, now: DateTime<Utc>, force: bool) -> anyhow::Result<()>; }`; `pub struct DeliveryHub<W: Write> { terminal: TerminalNotifier<W>, matrix: Option<Box<dyn RemoteDelivery>> }`. Implement async methods `DeliveryHub::send_notifications(&mut self, notifications: &[Notification]) -> anyhow::Result<Vec<DeliveryWarning>>` and `DeliveryHub::publish_status(&mut self, status: &str, now: DateTime<Utc>, force: bool) -> Vec<DeliveryWarning>`, and require every `main.rs` caller to `.await` them. Terminal delivery must still use `TerminalNotifier`/`render_notification_line` and return `Err` if stdout write fails. Matrix delivery failures are converted into `DeliveryWarning` and do not return `Err`. Warnings returned from Matrix best-effort failures are printed by `main.rs` as `Warning: ...` using `line_safe`. Update `src/main.rs` to route returned notifications through delivery in watch and replay, with replay constructing `DeliveryHub::terminal_only(...)`. Preserve terminal status rendering behavior.
  **Must NOT do**: Do not implement Matrix SDK sends yet; do not swallow terminal write failures.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: async orchestration boundary and runtime output behavior.
  - Skills: [] - Core Rust architecture.
  - Omitted: [`matrix-sdk`] - Matrix transport comes later.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: Tasks 8, 10, 12 | Blocked By: Tasks 2, 5

  **References**:
  - Pattern: `src/terminal.rs:69` - terminal notification write path.
  - Pattern: `src/terminal.rs:104` - `TerminalNotifier` currently implements sync notifier.
  - Pattern: `src/main.rs:172` - preload processing currently dispatches through monitor.
  - Pattern: `src/main.rs:218` - live records currently call `monitor.process_event`.
  - Pattern: `src/main.rs:280` - terminal live status render seam.
  - Pattern: `src/main.rs:310` - replay should use terminal-only delivery.

  **Acceptance Criteria**:
  - [ ] `cargo test --test terminal_rendering` passes.
  - [ ] `cargo test --test replay` passes.
  - [ ] `cargo test --test cli_config cli_config_watch_preloads_existing_event_output` passes.
  - [ ] `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` output remains newline-safe and contains expected replay fragments.

  **QA Scenarios**:
  ```
  Scenario: Replay remains terminal-only through delivery layer
    Tool: Bash
    Steps: cargo test --test replay
    Expected: Exit 0; replay tests assert expected stdout and do not require Matrix.
    Evidence: .omo/evidence/task-7-replay-terminal-only.txt

  Scenario: Terminal rendering unchanged
    Tool: Bash
    Steps: cargo test --test terminal_rendering
    Expected: Exit 0; terminal line safety and status rendering tests remain green.
    Evidence: .omo/evidence/task-7-terminal-rendering.txt
  ```

  **Commit**: CONDITIONAL | Message: `Route monitor output through delivery hub` | Files: [`src/delivery.rs`, `src/lib.rs`, `src/main.rs`, `src/terminal.rs`, `tests/replay.rs`, `tests/terminal_rendering.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 8. Async watch shutdown and status cadence plumbing

  **What to do**: Add watch-loop status cadence plumbing before Matrix status implementation. Define a reusable `StatusCadence` or `MatrixStatusSchedule` with default interval from Matrix config (`60s`) and methods to determine due/force updates. In `run_watch`, keep terminal status rendering every poll as today, but call and `.await` the delivery hub's async status publish with `force=false` after each poll and with `force=true` when monitor starts/stops if a Matrix sink exists later. Add `tokio::signal::ctrl_c` handling only if it can be done without changing existing test behavior; otherwise document in code that final status on Ctrl+C is out of scope for MVP and only normal `finish`/testable paths force status. Ensure malformed line/config warnings remain stderr-only, not Matrix notifications.
  **Must NOT do**: Do not add durable status event persistence or real Matrix edit code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: watch loop behavior and timing are delicate.
  - Skills: [] - No external skill needed.
  - Omitted: [`debugging`] - Not a bug investigation unless watch tests hang.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: Task 10 | Blocked By: Task 7

  **References**:
  - Pattern: `src/main.rs:208` - current status render after preload/start.
  - Pattern: `src/main.rs:230` - warnings checked each watch poll.
  - Pattern: `src/main.rs:235` - current status render after each poll.
  - Pattern: `src/terminal.rs:165` - deterministic status rendering.
  - User decision: Matrix status uses one editable message with periodic edit.

  **Acceptance Criteria**:
  - [ ] `cargo test --test live_tail live_tail_poll_interval_uses_runtime_config` passes.
  - [ ] New status cadence unit tests prove first update due, interval not due before 60s, due after 60s, and force always due.
  - [ ] `cargo test --test cli_config cli_config_watch_tails_until_stopped` passes without timing flakiness.

  **QA Scenarios**:
  ```
  Scenario: Status cadence does not spam Matrix sink
    Tool: Bash
    Steps: cargo test --lib delivery::tests::status_cadence_respects_interval
    Expected: Exit 0; fake clock checks one initial due update and no repeated update before interval.
    Evidence: .omo/evidence/task-8-status-cadence.txt

  Scenario: Watch loop remains responsive
    Tool: Bash
    Steps: cargo test --test cli_config cli_config_watch_tails_until_stopped
    Expected: Exit 0; async watch loop remains stoppable and does not hang.
    Evidence: .omo/evidence/task-8-watch-responsive.txt
  ```

  **Commit**: CONDITIONAL | Message: `Add async status cadence plumbing` | Files: [`src/delivery.rs`, `src/main.rs`, `tests/live_tail.rs`, `tests/cli_config.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 9. Matrix SDK delivery implementation with fakeable sender

  **What to do**: Implement `src/matrix.rs` using `matrix-sdk`. Start with a compile-proof SDK skeleton: add imports for `Client`, `MatrixSession`, `SessionMeta`, `SessionTokens`, `SyncSettings`, `RoomMessageEventContent`, `Mentions`, and edit helpers, then run `cargo check --all-targets` before integrating behavior. Production setup: build `Client` with `homeserver`, restore session with `MatrixSession { SessionMeta { user_id, device_id: EDAFKDASHBOARD }, SessionTokens { access_token, refresh_token: None } }`, wrap client build/session restore/`sync_once`/room lookup in `tokio::time::timeout(Duration::from_secs(10), ...)`, run `sync_once(SyncSettings::default()).await`, get configured room by `room_id`, and return a Matrix sender. Implement plain notification send with `RoomMessageEventContent::text_plain(body)` and `room.send(content).await`, wrapped in `timeout(Duration::from_secs(5), ...)`. Build body as optional mention prefix + optional emoji + `remote_text`, line-sanitized and without raw payloads. For mentions, parse `mention_user_id`, add readable prefix, and set `Mentions::with_user_ids([mentioned_user])`; if no `mention_user_id`, send normal Matrix text even when `level >= 2`. Implement status create and edit: first status call sends text and stores original event ID; later due/forced calls use `room.make_edit_event(&original_status_event_id, EditedContent::RoomMessage(RoomMessageEventContentWithoutRelation::text_plain(status))).await` and `room.send(edit_event).await`, each wrapped in 5-second timeout, while retaining the original event ID. Implement `MatrixDelivery` as the production `RemoteDelivery`, and add `FakeRemoteDelivery` in tests so normal tests do not instantiate `matrix-sdk` network clients. Do not detect E2EE; docs already state unsupported.
  **Must NOT do**: Do not run a long Matrix sync loop, receive commands, support multiple rooms, persist session storage, or print token in errors.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: external SDK integration with async/fakeable architecture.
  - Skills: [] - Matrix details are already researched in this plan.
  - Omitted: [`notion-api`] - Not related.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: Tasks 10, 12 | Blocked By: Tasks 1, 3, 6

  **References**:
  - External: `matrix-sdk 0.18.0 docs.rs` - `Client`, `authentication::matrix::MatrixSession`, `SessionMeta`, `SessionTokens`, `config::SyncSettings`.
  - External: `matrix_sdk::ruma::events::room::message::RoomMessageEventContent` - `text_plain` content constructor.
  - External: `matrix_sdk::ruma::events::Mentions` - `Mentions::with_user_ids` for Matrix mention metadata.
  - External: `matrix_sdk::room::edit::EditedContent` and `Room::make_edit_event` - status edit helper.
  - Pattern: `src/notifier.rs:12` - `Notification.remote_text`, `emoji`, `level`, and `mention` are Matrix input.
  - Pattern: `src/text.rs` - use `line_safe` to avoid control/newline injection in Matrix messages.

  **Acceptance Criteria**:
  - [ ] `cargo check --all-targets` passes immediately after adding the Matrix SDK skeleton imports/types.
  - [ ] `cargo test --lib matrix::tests::matrix_formats_level_one_without_mention` passes.
  - [ ] `cargo test --lib matrix::tests::matrix_formats_level_two_with_mentions_metadata` passes.
  - [ ] `cargo test --lib matrix::tests::matrix_status_edits_original_event_id` passes.
  - [ ] `cargo test --lib matrix::tests::matrix_errors_redact_access_token` passes.
  - [ ] `cargo test --all` passes without network access.

  **QA Scenarios**:
  ```
  Scenario: Level two Matrix message includes real mention metadata
    Tool: Bash
    Steps: cargo test --lib matrix::tests::matrix_formats_level_two_with_mentions_metadata
    Expected: Exit 0; fake sender receives one body with readable user prefix and structured mention user ID.
    Evidence: .omo/evidence/task-9-matrix-mention.txt

  Scenario: Status edits retain original event id
    Tool: Bash
    Steps: cargo test --lib matrix::tests::matrix_status_edits_original_event_id
    Expected: Exit 0; fake sender records one create and later edit(s) referencing the original status event id, not edit event ids.
    Evidence: .omo/evidence/task-9-status-edit.txt

  Scenario: Matrix SDK compile proof succeeds before behavior integration
    Tool: Bash
    Steps: cargo check --all-targets
    Expected: Exit 0 after adding Matrix SDK imports, session-restore skeleton, and fakeable sender interfaces.
    Evidence: .omo/evidence/task-9-sdk-compile-proof.txt
  ```

  **Commit**: CONDITIONAL | Message: `Add Matrix SDK delivery backend` | Files: [`Cargo.toml`, `Cargo.lock`, `src/matrix.rs`, `src/lib.rs`, `src/text.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 10. Watch-mode Matrix integration and replay isolation

  **What to do**: Wire Matrix runtime setup into `run_watch` only. Build terminal delivery always. If `RuntimeConfig.matrix` validates and current mode is Watch, attempt `MatrixDelivery::connect(...).await` with the Task 9 10-second startup timeout policy; if connection/session/`sync_once`/room lookup times out or fails, print `Warning: Matrix delivery disabled: <redacted reason>` and continue terminal-only. If connection succeeds, attach Matrix sink to delivery hub. Process preload, reset-session notification, monitor_started, live records, warnings, terminal status, and Matrix status through the same delivery hub, awaiting `DeliveryHub::send_notifications(...).await` and `DeliveryHub::publish_status(...).await` at every call site. Ensure every `Notification` returned from `EventMonitor` in watch is sent to terminal and Matrix according to level. Runtime Matrix send/edit timeout or failure must return `DeliveryWarning` and never abort terminal delivery. In `run_replay`, do not call any Matrix validation/connect/send/edit path; use terminal-only delivery even when config contains Matrix. Update startup text to report Matrix disabled/enabled/unavailable accurately and remove Discord-webhook wording.
  **Must NOT do**: Do not add `--matrix`, `--no-matrix`, Matrix command handling, or Matrix in replay.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: cross-cutting integration across config, delivery, watch, replay, tests.
  - Skills: [] - Core implementation.
  - Omitted: [`playwright`] - No UI/browser.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: Task 12 | Blocked By: Tasks 5, 7, 8, 9

  **References**:
  - Pattern: `src/main.rs:144` - watch path must build Matrix only here.
  - Pattern: `src/main.rs:172` - preload records are currently processed before live tail.
  - Pattern: `src/main.rs:184` - reset-session emits a notification.
  - Pattern: `src/main.rs:201` - monitor start notification seam.
  - Pattern: `src/main.rs:310` - replay must stay Matrix-free.
  - Pattern: `src/main.rs:384` - startup text currently has old Discord wording.
  - Test: `tests/replay.rs` - add fake Matrix guard proving no Matrix side effects.
  - Test: `tests/cli_config.rs` - watch binary behavior remains stable.

  **Acceptance Criteria**:
  - [ ] New test `replay_matrix_config_does_not_initialize_matrix` passes.
  - [ ] New test `watch_matrix_init_failure_falls_back_to_terminal` passes.
  - [ ] New test `watch_level_one_and_two_notifications_reach_fake_matrix` passes.
  - [ ] New test `watch_delayed_matrix_send_warns_without_blocking_terminal_output` passes.
  - [ ] `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` exits `0` with Matrix config present in test fixture working dir and no Matrix sends.
  - [ ] `rg -n "Discord webhook" README.md config.example.toml src tests` returns no stale user-facing Discord startup wording.

  **QA Scenarios**:
  ```
  Scenario: Replay ignores Matrix config by construction
    Tool: Bash
    Steps: cargo test --test replay replay_matrix_config_does_not_initialize_matrix
    Expected: Exit 0; fake Matrix factory records zero connect attempts and replay stdout remains terminal-only.
    Evidence: .omo/evidence/task-10-replay-no-matrix.txt

  Scenario: Matrix init failure does not stop watch terminal output
    Tool: Bash
    Steps: cargo test --test cli_config watch_matrix_init_failure_falls_back_to_terminal
    Expected: Exit 0; stderr contains one Matrix disabled warning and stdout still contains terminal notification output.
    Evidence: .omo/evidence/task-10-matrix-fallback.txt

  Scenario: Delayed Matrix send is timed out while terminal output continues
    Tool: Bash
    Steps: cargo test --test cli_config watch_delayed_matrix_send_warns_without_blocking_terminal_output
    Expected: Exit 0; fake delayed Matrix sink triggers timeout warning and terminal notification still appears.
    Evidence: .omo/evidence/task-10-matrix-timeout.txt
  ```

  **Commit**: CONDITIONAL | Message: `Enable Matrix delivery in watch mode` | Files: [`src/main.rs`, `src/delivery.rs`, `src/matrix.rs`, `tests/replay.rs`, `tests/cli_config.rs`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 11. README/config examples and knowledge update after integration

  **What to do**: After code integration is stable, update README with verified CLI commands and expected behavior. Add a knowledge file `.omo/knowledges/matrix-bot-async-delivery.md` summarizing final architecture: async main, producer monitor, delivery hub, Matrix SDK access-token restore, no E2EE, replay isolation, config.toml gitignore, and level semantics. Ensure README includes copy-pastable commands: `cp config.example.toml config.toml`, `cargo run -- --journal "/path/to/Elite Dangerous"`, explicit `--config`, replay no-Matrix example, and verification commands. Run secret scan over docs/knowledge changes before commit.
  **Must NOT do**: Do not include real Matrix token, real room ID, private user ID beyond placeholders, or config.local.toml references.

  **Recommended Agent Profile**:
  - Category: `writing` - Reason: final docs and project knowledge.
  - Skills: [`secret-guard`] - Docs mention tokens.
  - Omitted: [`frontend-claude`] - No UI design.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: Task 12 | Blocked By: Task 10

  **References**:
  - Pattern: `README.md:28` - Journal paths and run examples.
  - Pattern: `README.md:92` - configuration section.
  - Pattern: `README.md:131` - privacy/fixtures section.
  - Pattern: `.omo/knowledges/task-2-cli-config-contract.md` - config contract knowledge style.
  - Pattern: `.omo/knowledges/task-5-notifier-dispatcher.md` - notifier knowledge style.
  - Rule: Repo instruction says valuable knowledge should be saved under `.omo/knowledges/*.md`.

  **Acceptance Criteria**:
  - [ ] `python "/home/ubuntu/.config/opencode/skills/secret-guard/scripts/scan_secrets.py" staged` exits `0` after staging docs/knowledge in implementation.
  - [ ] README contains `cp config.example.toml config.toml` and does not contain `config.local.toml`.
  - [ ] Knowledge file exists at `.omo/knowledges/matrix-bot-async-delivery.md` and mentions replay isolation and no E2EE.
  - [ ] `cargo test --all` passes after docs/knowledge changes.

  **QA Scenarios**:
  ```
  Scenario: Documentation contains only supported config file names
    Tool: Bash
    Steps: python - <<'PY'
from pathlib import Path
text=Path('README.md').read_text()+Path('config.example.toml').read_text()
assert 'config.toml' in text
assert 'config.local.toml' not in text
assert 'access_token_env' not in text
PY
    Expected: Exit 0; docs mention only config.toml/config.example.toml and no env-token field.
    Evidence: .omo/evidence/task-11-doc-config-names.txt

  Scenario: Knowledge captures final architecture
    Tool: Bash
    Steps: python - <<'PY'
from pathlib import Path
p=Path('.omo/knowledges/matrix-bot-async-delivery.md')
text=p.read_text()
for needle in ['async', 'producer', 'Matrix', 'replay', 'E2EE']:
    assert needle in text
PY
    Expected: Exit 0; knowledge file includes final architecture keywords.
    Evidence: .omo/evidence/task-11-knowledge.txt
  ```

  **Commit**: CONDITIONAL | Message: `Document Matrix delivery architecture` | Files: [`README.md`, `config.example.toml`, `.omo/knowledges/matrix-bot-async-delivery.md`] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

- [x] 12. Full regression hardening and release-readiness QA

  **What to do**: Run the full validation suite, fix failures within scope, and add any missing regression tests revealed by failures. Required commands: `cargo fmt --check`, `cargo test --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo run -- --help`, `cargo run -- --replay --no-status-line`, and `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line`. Add a test fixture config with fake Matrix token only under test temp dirs; never commit real `config.toml`. Confirm `config.toml` is ignored and not tracked. Confirm no stale `level 3 = mention` docs remain except compatibility tests explicitly saying `3+` is mention. Confirm no raw Journal payload is sent to Matrix by searching Matrix formatter tests and code paths.
  **Must NOT do**: Do not run real Matrix credentials unless explicitly provided outside this plan; do not add network-dependent CI tests.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: cross-suite hardening and regression cleanup.
  - Skills: [`secret-guard`] - Final leak scan required.
  - Omitted: [`playwright`] - No browser UI.

  **Parallelization**: Can Parallel: NO | Wave 4 | Blocks: final verification | Blocked By: Tasks 5, 10, 11

  **References**:
  - Pattern: `README.md:60` - current normal test command.
  - Pattern: `README.md:72` - expected replay signals.
  - Pattern: `tests/replay.rs` - replay integration regressions.
  - Pattern: `tests/fixtures/README.md` - fixture privacy policy.
  - Pattern: `.gitignore:10` - local/private input ignores.

  **Acceptance Criteria**:
  - [ ] `cargo fmt --check` exits `0`.
  - [ ] `cargo test --all` exits `0`.
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings` exits `0`.
  - [ ] `cargo run -- --help` exits `0`.
  - [ ] `cargo run -- --replay --no-status-line` exits non-zero and prints `Error: replay requires --set-file <file>`.
  - [ ] `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` exits `0`.
  - [ ] `git check-ignore config.toml` exits `0`.
  - [ ] Secret scan of staged files exits `0` before any commit.

  **QA Scenarios**:
  ```
  Scenario: Full Rust quality gate passes
    Tool: Bash
    Steps: cargo fmt --check && cargo test --all && cargo clippy --all-targets --all-features -- -D warnings
    Expected: All commands exit 0 with no formatting, test, or lint failures.
    Evidence: .omo/evidence/task-12-full-quality.txt

  Scenario: Replay remains Matrix-free and functional
    Tool: Bash
    Steps: cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
    Expected: Exit 0; output contains terminal replay summary fragments and test suite separately proves no Matrix init/send in replay.
    Evidence: .omo/evidence/task-12-replay-qa.txt
  ```

  **Commit**: CONDITIONAL | Message: `Harden Matrix delivery regressions` | Files: [all touched source/test/doc files from final fixes] | Only commit if the user explicitly requests commits; otherwise leave changes uncommitted and report touched files.

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.
- [x] F1. Plan Compliance Audit — oracle
- [x] F2. Code Quality Review — unspecified-high
- [x] F3. Agent-Executed CLI/Matrix E2E QA — unspecified-high (`cargo run -- --help`, replay success/error commands, fake Matrix watch tests, evidence in `.omo/evidence/f3-matrix-e2e.txt`)
- [x] F4. Scope Fidelity Check — deep

## Commit Strategy
- If and only if the user explicitly requests commits, use multiple small commits because this is a large refactor. Recommended commit sequence follows task boundaries:
  1. `Support local Matrix config loading`
  2. `Update notification routing levels`
  3. `Run CLI orchestration on Tokio`
  4. `Make monitor produce notifications`
  5. `Route monitor output through delivery hub`
  6. `Add Matrix SDK delivery backend`
  7. `Enable Matrix delivery in watch mode`
  8. `Document Matrix delivery architecture`
  9. `Harden Matrix delivery regressions`
- Before each requested commit that touches config/docs/Matrix code, run staged secret scan: `python "/home/ubuntu/.config/opencode/skills/secret-guard/scripts/scan_secrets.py" staged`.
- If commits are not requested, do not commit; report touched files and verification results only.
- Never commit `config.toml`; it must be gitignored.

## Success Criteria
- Watch mode can run one executable with terminal output and optional Matrix output from the same notification stream.
- Replay mode is terminal-only regardless of Matrix config.
- Matrix messages mirror business notifications produced by `EventMonitor`; status is represented by one editable Matrix message.
- Level semantics are simple and documented: `0 = off`, `1 = notify`, `2+ = notify + mention`.
- Matrix SDK integration is testable without real network credentials.
- Matrix token is never logged, committed, or included in examples except as `<token>`.
