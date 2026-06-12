# ed-afk-monitor Phase 1: Rust CLI AFK Journal Monitor

## TL;DR
> **Summary**: Initialize an independent Rust CLI named `ed-afk-monitor` that tails/replays Elite Dangerous Journal files, maintains AFK session state, renders terminal event/status output, and emits structured notifications through a Matrix-ready abstraction without implementing Matrix yet.
> **Deliverables**:
> - Rust crate/binary `ed-afk-monitor` with clean module boundaries.
> - TDD parser/state/notifier/tail/replay/warning tests using sanitized fixtures plus optional read-only real Journal replay.
> - Phase 1 CLI: default watch mode plus top-level `--replay`, with flags `--journal`, `--set-file`, `--file-select`, `--reset-session`, `--debug`.
> - TOML config with requested `[journal]`, `[monitor]`, and `[log_levels]` defaults.
> - Terminal event logs and single-line live status for TTY; newline-safe non-TTY output.
> - GitHub Actions CI and release artifacts for Windows x64 and Linux x64.
> **Effort**: Large
> **Parallel**: YES - 3 implementation waves + final verification wave
> **Critical Path**: Task 1 → Task 3 → Task 10 → Task 11 → Task 12 → Task 14 → Final Verification

## Context
### Original Request
Build a lightweight Rust-based Elite Dangerous AFK session monitor inspired by `PsiPab/ED-AFK-Monitor` behavior, with Phase 1 CLI parity and Phase 2 Matrix delivery later. The project must remain independent: no fork, no copied Python code structure/function names/message text/README prose/Discord logic.

### Interview Summary
- Scope: **Phase 1 first**. Matrix remains deferred, but Phase 1 must include `Notification` and `Notifier` abstractions.
- Testing: **TDD** with unit tests and real-data simulation. Raw local Journal files under `/home/ubuntu/Elite Dangerous` are read-only test inputs and must not be committed.
- Packaging: include GitHub Actions for Windows x64 and Linux x64 release artifacts.
- Runtime target: Windows primary, Linux supported.
- Repo state: empty/new project; no `Cargo.toml`, `src/`, tests, docs, CI, commits, or project `AGENTS.md` existed before planning artifacts.

### Metis Review (gaps addressed)
- Locked crate/package/binary/artifact prefix to `ed-afk-monitor`.
- Locked CLI contract, exit codes, discovery ordering, preload/live/replay semantics, warning timing, session-start rules, kill semantics, duplicate key, TTY behavior, and artifact names.
- Added clean-room guardrails, sanitized-fixture policy, deterministic clock requirements, and no-human-QA acceptance criteria.

## Work Objectives
### Core Objective
Create a Phase 1 Rust CLI monitor that can discover or accept Journal files, preload state without dispatching notifications, replay deterministic logs, live-tail a selected file by polling, parse the requested Phase 1 events, maintain AFK session statistics, emit terminal notifications/status lines, and pass automated tests/CI on Linux and Windows.

### Deliverables
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `.gitignore`.
- `src/main.rs`, `src/config.rs`, `src/journal.rs`, `src/event.rs`, `src/state.rs`, `src/monitor.rs`, `src/terminal.rs`, `src/notifier.rs`, `src/text.rs`, `src/time.rs`; `src/matrix.rs` may exist only as a non-wired placeholder module comment or be omitted entirely in Phase 1.
- `tests/fixtures/*.log` sanitized/minimal Journal fixtures.
- `tests/*.rs` parser/state/notifier/replay/live-tail/CLI/integration tests.
- `config.example.toml`, `README.md` with Phase 1 usage and Phase 2 roadmap only.
- `.github/workflows/ci.yml` and `.github/workflows/release.yml`.

### Definition of Done (verifiable conditions with commands)
Run from repo root:
```bash
cargo metadata --format-version 1
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
cargo test --test real_journal_replay -- --ignored
```
Expected signals:
- Metadata command prints valid Cargo JSON and exits `0`.
- fmt/clippy/tests exit `0`.
- Replay command exits `0` and prints newline-safe upstream-style event output containing `Scan`, `Kill`, `Total Stats`, and `Monitor stopped` fragments.
- Ignored real replay test exits `0` only when `/home/ubuntu/Elite Dangerous` exists; otherwise it prints a skip message and exits `0`.

### Must Have
- Clean-room implementation; upstream repo is only behavior-level inspiration.
- Windows default Journal directory: `%USERPROFILE%\Saved Games\Frontier Developments\Elite Dangerous`.
- Linux/dev explicit Journal path: `--journal /home/ubuntu/Elite Dangerous` or `--set-file <file>`.
- Journal glob: `Journal.*.log`; newest by parsed filename timestamp, then mtime fallback.
- Phase 1 no journal rotation; tail only selected/latest file.
- Preload builds state and does not dispatch notifications; `--reset-session` clears counters after preload; live starts at EOF; replay reads whole file and exits.
- Events to parse: `Commander`, `LoadGame`, `Loadout`, `Rank`, `Progress`, `Location`, `SupercruiseDestinationDrop`, `SupercruiseEntry`, `FSDJump`, `ReceiveText`, `ShipTargeted`, `Bounty`, `FactionKillBond`, `MissionRedirected`, `Missions`, `MissionAccepted`, `MissionCompleted`, `MissionFailed`, `MissionAbandoned`, `ShieldState`, `HullDamage`, `FighterDestroyed`, `LaunchFighter`, `EjectCargo`, `ReservoirReplenished`, `Music`, `Shutdown`, `Died`.
- Observed kills count `Bounty` and `FactionKillBond`; massacre mission progress is separate.
- Warnings disabled during preload; replay warning time uses Journal timestamps; live warning time uses injected/system clock.
- Default warning thresholds: `warn_kill_rate = 20`, `warn_no_kills_initial_minutes = 5`, `warn_no_kills_minutes = 20`, `warn_cooldown_minutes = 30`.
- Duplicate suppression: consecutive same `event_type` + same `terminal_text`, suppress after `duplicate_max = 5`, in dispatcher only; after the fifth duplicate, emit exactly one notification with event_type `duplicate_suppression` and text fragment `Suppressing further duplicate messages`, then suppress until event type or text changes.
- Terminal status: crossterm single-line for TTY; no control characters in non-TTY/CI.

### Locked CLI Contract
- There are no mode subcommands. Global flags are top-level options: `--journal <folder>`, `--set-file <file>`, `--file-select`, `--reset-session`, `--debug`, `--config <file>`, `--no-status-line`, `--replay`.
- `--poll-interval-ms <ms>` is watch-only; it is rejected when `--replay` is present.
- If `--replay` is absent, the invocation runs watch mode.
- Exact valid examples:
  ```bash
  ed-afk-monitor --journal "/home/ubuntu/Elite Dangerous"
  ed-afk-monitor --journal "/home/ubuntu/Elite Dangerous" --poll-interval-ms 1000
  ed-afk-monitor --debug --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
  ed-afk-monitor --set-file tests/fixtures/journal_combat_bounty.log --replay --no-status-line
  ```
- Exact invalid examples:
  ```bash
  ed-afk-monitor --replay --poll-interval-ms 1000
  ed-afk-monitor --replay --journal "/home/ubuntu/Elite Dangerous"
  ```
  `--replay` requires `--set-file <file>` and rejects `--journal` unless a future version adds folder replay.
- `--reset-session` applies only to `watch`: preload selected file, then clear counters before live processing. In `replay`, `--reset-session` is accepted as a global compatibility flag but ignored with one warning containing `--reset-session has no effect in replay`.

### Locked Config Contract
```toml
[journal]
folder = ""
recent_files = 10

[monitor]
use_utc = false
live_status = true
warn_kill_rate = 20
warn_no_kills_minutes = 20
warn_no_kills_initial_minutes = 5
warn_cooldown_minutes = 30
duplicate_max = 5
pirate_names = false
bounty_faction = false
bounty_value = false
extended_stats = false
min_scan_level = 1
poll_interval_ms = 1000

[log_levels]
scan_incoming = 1
scan_easy = 1
scan_hard = 2
kill_easy = 2
kill_hard = 2
fighter_hull = 2
fighter_down = 3
ship_shields = 3
ship_hull = 3
died = 3
cargo_lost = 3
bait_value_low = 2
security_scan = 2
security_attack = 3
fuel_report = 1
fuel_low = 2
fuel_critical = 3
missions = 2
missions_all = 3
no_kills = 3
kill_rate = 3
summary_kills = 2
summary_faction = 0
summary_scans = 0
summary_bounties = 2
duplicate_suppression = 1
```
- Config precedence: CLI > `--config <file>` TOML > defaults.
- No environment-variable config in Phase 1 except reading `USERPROFILE` to resolve the Windows default Journal directory.
- Missing config keys use defaults; wrong-typed keys produce a warning and use defaults for that key; malformed TOML exits `1`.

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)
- Do not copy upstream Python code, code structure, unique function names, comments, messages, README prose, or Discord handling.
- Do not implement Matrix, Discord, WebUI, EDMC plugin, command bot, database, historical dashboard, auto-relog, key simulation, or game automation in Phase 1.
- Do not commit raw Journal files from `/home/ubuntu/Elite Dangerous`.
- Do not require manual visual confirmation for acceptance criteria.
- Do not add full-screen TUI/ratatui/curses; only simple status line.
- Do not parse the full Elite Journal catalog beyond listed Phase 1 events.
- Do not make Matrix-specific config mandatory in Phase 1.

## Verification Strategy
> ZERO HUMAN INTERVENTION - all verification is agent-executed.
- Test decision: TDD + Rust `cargo test`; sanitized fixtures in CI; optional ignored real Journal replay test.
- QA policy: Every task has agent-executed happy and failure/edge scenarios.
- Evidence: `.omo/evidence/task-{N}-{slug}.{ext}`.

## Execution Strategy
### Parallel Execution Waves
> Target: 5-8 tasks per wave. Shared dependencies are Wave 1 tasks for max parallelism.

Wave 1: Tasks 1-6 — project/tooling, CLI/config contract tests, parser fixtures, time/stat primitives, notifier/dispatcher, sanitized fixture policy.
Wave 2: Tasks 7-12 — Journal discovery/preload, replay, live tail, state/mission tracking, event monitor notifications, warning scheduler.
Wave 3: Tasks 13-16 — terminal rendering, integration/real replay, docs/config, CI/release artifacts.

### Dependency Matrix (full, all tasks)
- T1 blocks all tasks.
- T2 depends on T1; blocks T7, T8, T9, T14, T16.
- T3 depends on T1 and T6; blocks T8, T10, T11, T14.
- T4 depends on T1; blocks T10 and T12.
- T5 depends on T1; blocks T11, T12, T13.
- T6 depends on T1; blocks T3 and T14.
- T7 depends on T2; blocks T8 and T9.
- T8 depends on T2, T3, T7; blocks T14.
- T9 depends on T2, T7; blocks T14.
- T10 depends on T3 and T4; blocks T11 and T12.
- T11 depends on T3, T5, T10; blocks T13 and T14.
- T12 depends on T4, T5, T10, T11; blocks T14.
- T13 depends on T5, T11, T12; blocks T14.
- T14 depends on T8, T9, T11, T12, T13.
- T15 depends on T2, T7, T8, T13.
- T16 depends on T1, T14, T15.

### Agent Dispatch Summary (wave → task count → categories)
- Wave 1 → 6 tasks → quick x1, unspecified-high x5.
- Wave 2 → 6 tasks → unspecified-high x5, deep x1.
- Wave 3 → 4 tasks → visual-engineering x1, unspecified-high x2, writing x1.
- Dispatch categories are from the current runner's supported set: `quick`, `unspecified-high`, `deep`, `visual-engineering`, `writing`.

## TODOs
> Implementation + Test = ONE task. Never separate.
> EVERY task MUST have: Agent Profile + Parallelization + QA Scenarios.

- [x] 1. Initialize Rust project, toolchain, and baseline quality gates

  **What to do**: Create a fresh Rust binary crate named `ed-afk-monitor`. Add `Cargo.toml` with package/binary name `ed-afk-monitor`, dependencies `anyhow`, `clap` with derive, `chrono` with serde, `serde` derive, `serde_json`, `toml`, `tokio` rt-multi-thread/macros/fs/time, `reqwest` json/rustls-tls default-features=false, `crossterm`, `notify`, `uuid` v4, plus dev-dependencies `assert_cmd`, `predicates`, `tempfile`, and `insta` only if snapshot tests are used. Add `rust-toolchain.toml` pinned to stable, `rustfmt.toml`, `.gitignore`, empty module files, a minimal `main` that exits successfully for `--version`, and create local directory `.omo/evidence/` before any QA scenario writes evidence.
  **Must NOT do**: Do not implement monitor logic here. Do not add Matrix SDK. Do not create raw Journal fixtures. Do not commit generated `.omo/evidence/*` files.

  **Recommended Agent Profile**:
  - Category: `quick` - Reason: project skeleton and config files are straightforward.
  - Skills: [] - No specialized skill required.
  - Omitted: [`git-master`] - No commit requested inside this task.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [2-16] | Blocked By: []

  **References**:
  - Target: `Cargo.toml` - package and dependency contract.
  - Target: `src/main.rs` - binary entry point.
  - Target: `src/{config,journal,event,state,monitor,terminal,notifier,text,time}.rs` - module boundaries.
  - External: `https://doc.rust-lang.org/cargo/reference/manifest.html` - Cargo manifest contract.

  **Acceptance Criteria**:
  - [ ] `cargo metadata --format-version 1` exits `0`.
  - [ ] `cargo run -- --version` exits `0` and stdout contains `ed-afk-monitor`.
  - [ ] `cargo fmt --check` exits `0`.
  - [ ] `test -d .omo/evidence` exits `0`.

  **QA Scenarios**:
  ```
  Scenario: Fresh project builds metadata
    Tool: Bash
    Steps: cargo metadata --format-version 1 > .omo/evidence/task-1-metadata.json
    Expected: command exits 0; evidence file contains package name "ed-afk-monitor"
    Evidence: .omo/evidence/task-1-metadata.json

  Scenario: Unknown CLI flag fails through clap later-safe baseline
    Tool: Bash
    Steps: cargo run -- --definitely-unknown > .omo/evidence/task-1-unknown-flag.txt 2>&1; test $? -ne 0
    Expected: command returns non-zero and output mentions unexpected/unknown argument
    Evidence: .omo/evidence/task-1-unknown-flag.txt
  ```

  **Commit**: YES | Message: `chore(project): initialize rust crate` | Files: [`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`, `.gitignore`, `src/**`]

- [x] 2. Define CLI and TOML config contract with tests

  **What to do**: Implement `clap` CLI exactly as the **Locked CLI Contract** section states. Global flags accepted before/after subcommands: `--journal <folder>`, `--set-file <file>`, `--file-select`, `--reset-session`, `--debug`, `--config <file>`, `--no-status-line`. `--poll-interval-ms <ms>` is `watch`-only. If no subcommand is provided, treat the invocation as a compatibility alias for `watch`. `replay` requires `--set-file <file>` and rejects `--journal`. Implement config loading/merging in `src/config.rs` exactly as the **Locked Config Contract** section states: precedence CLI > config file > defaults. Add `config.example.toml` with the full locked schema/defaults. Exit code contract: 0 success, 1 runtime/config/journal errors, 2 clap errors, 3 reserved and unused unless strict replay is later added.
  **Must NOT do**: Do not silently create or write user config. Do not add Matrix config as active Phase 1 config; mention Phase 2 only in comments/docs if needed.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: cross-platform CLI/config precedence requires careful tests.
  - Skills: [] - No specialized skill required.
  - Omitted: [`secret-guard`] - No secrets are introduced; Matrix token is out of scope.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [7,8,9,14,16] | Blocked By: [1]

  **References**:
  - Target: `src/main.rs` - CLI entry and exit code mapping.
  - Target: `src/config.rs` - config structs/defaults/merge.
  - Target: `config.example.toml` - user-facing default config.
  - External: `https://docs.rs/clap/latest/clap/_derive/index.html` - clap derive patterns.
  - External: `https://docs.rs/toml/latest/toml/` - TOML parsing.

  **Acceptance Criteria**:
  - [ ] `cargo test cli_config --all` exits `0`.
  - [ ] `cargo run -- --help` exits `0` and contains `watch`, `replay`, `--journal`, `--set-file`, `--file-select`, `--reset-session`, `--debug`.
  - [ ] `cargo run -- --bad-flag` exits with clap error code `2`.
  - [ ] Config merge test proves CLI `--journal` overrides TOML `[journal].folder`.
  - [ ] `cargo run -- replay --poll-interval-ms 1000 --set-file tests/fixtures/journal_combat_bounty.log` exits with clap error code `2`.
  - [ ] `cargo test cli_config_no_subcommand_watch_alias --all` exits `0` and asserts argv `--set-file tests/fixtures/journal_combat_bounty.log --no-status-line` parses to the `watch` variant with expected global flags, without running live tail.

  **QA Scenarios**:
  ```
  Scenario: CLI exposes Phase 1 contract
    Tool: Bash
    Steps: cargo run -- --help > .omo/evidence/task-2-help.txt
    Expected: output includes watch, replay, --journal, --set-file, --file-select, --reset-session, --debug
    Evidence: .omo/evidence/task-2-help.txt

  Scenario: Malformed TOML returns runtime config error
    Tool: Bash
    Steps: printf '[monitor\n' > /tmp/opencode/ed-afk-monitor-bad.toml; cargo run -- --config /tmp/opencode/ed-afk-monitor-bad.toml watch > .omo/evidence/task-2-bad-config.txt 2>&1; test $? -eq 1
    Expected: exit code 1; output mentions config parse failure without panic backtrace
    Evidence: .omo/evidence/task-2-bad-config.txt
  ```

  **Commit**: YES | Message: `feat(cli): define phase one command contract` | Files: [`src/main.rs`, `src/config.rs`, `config.example.toml`, `tests/**`]

- [x] 3. Implement Journal event parser with field-level tests

  **What to do**: Implement `src/event.rs` with a `JournalEvent` enum/struct model that preserves timestamp and event name and parses the listed Phase 1 events. Use serde structs with optional fields for data that Journal may omit. Unknown events parse to `JournalEvent::Unknown { event, timestamp }`; malformed JSON returns a recoverable parse error. Required fields: all parsed events require `timestamp` and `event`; `Bounty` uses optional `TotalReward`, `VictimFaction`, `Target`; `FactionKillBond` uses optional reward/faction fields; `ShipTargeted` uses optional `TargetLocked`, `ScanStage`, `PilotName`, `LegalStatus`; `ReceiveText` uses optional `From`, `From_Localised`, `Message`, `Channel`; `Mission*` events use optional `MissionID`, `Name`, `LocalisedName`; `ShieldState` uses optional `ShieldsUp`; `HullDamage` uses optional `Health`, `PlayerPilot`, `Fighter`; `EjectCargo` uses optional `Type`, `Count`, `Abandoned`; `ReservoirReplenished` uses optional fuel fields; `Music` uses optional `MusicTrack`.
  **Must NOT do**: Do not fail replay on unknown events. Do not copy event handling code from upstream.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: parser must handle many variants and malformed data safely.
  - Skills: [] - No specialized skill required.
  - Omitted: [`librarian`] - Behavior research is already available; no further external research needed unless ED schema ambiguity blocks implementation.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [8,10,11,14] | Blocked By: [1,6]

  **References**:
  - Target: `src/event.rs` - parser and event types.
  - Target: `tests/fixtures/journal_minimal_start.log` - Commander/LoadGame/Location/Supercruise entries.
  - Target: `tests/fixtures/journal_combat_bounty.log` - ReceiveText/ShipTargeted/Bounty/FactionKillBond.
  - External: `https://github.com/PsiPab/ED-AFK-Monitor/blob/6d11651f2992d801ebb33cf81a9fd35b01354244/README.md#L103-L117` - behavior-level Journal caveats only.

  **Acceptance Criteria**:
  - [ ] `cargo test event_parser --all` exits `0`.
  - [ ] Tests cover every Phase 1 event name at least once.
  - [ ] Tests cover unknown event, malformed JSON, missing optional fields, and missing `event` field.

  **QA Scenarios**:
  ```
  Scenario: All Phase 1 fixture lines parse or recover
    Tool: Bash
    Steps: cargo test event_parser --all > .omo/evidence/task-3-parser-tests.txt
    Expected: parser tests pass; unknown events are not fatal; malformed JSON test returns recoverable error
    Evidence: .omo/evidence/task-3-parser-tests.txt

  Scenario: Malformed line does not panic
    Tool: Bash
    Steps: cargo test event_parser_malformed_json --all > .omo/evidence/task-3-malformed.txt
    Expected: test passes and asserts no panic plus recoverable parse error
    Evidence: .omo/evidence/task-3-malformed.txt
  ```

  **Commit**: YES | Message: `feat(event): parse phase one journal events` | Files: [`src/event.rs`, `tests/**`, `tests/fixtures/**`]

- [x] 4. Implement deterministic time/statistics primitives

  **What to do**: Implement `src/time.rs` and state-adjacent helpers for duration formatting, local/UTC time display, rate calculations, rolling recent-rate windows, and an injectable clock trait or test clock. Rate formula: total kills/hour = `kills / max(session_duration_hours, 1 second as hours)` once session is active; recent average uses last 10 minutes of kill/scan timestamps; no active session returns `0.0/h` and suppresses warnings. Format durations as `58s`, `12m`, `3h12m`.
  **Must NOT do**: Do not use wall-clock directly inside warning/state tests; route through injected clock or explicit Journal timestamps.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: warning correctness depends on deterministic time.
  - Skills: [] - No specialized skill required.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [10,12] | Blocked By: [1]

  **References**:
  - Target: `src/time.rs` - formatting and clock abstraction.
  - Target: `src/state.rs` - rate calculation helpers may live here if cleaner.
  - External: `https://docs.rs/chrono/latest/chrono/` - DateTime and duration handling.

  **Acceptance Criteria**:
  - [ ] `cargo test time_stats --all` exits `0`.
  - [ ] Tests verify total and recent rates for zero duration, 1 hour, and 10-minute rolling window.
  - [ ] Tests verify UTC/local formatting is deterministic when configured.

  **QA Scenarios**:
  ```
  Scenario: Rate calculations are deterministic
    Tool: Bash
    Steps: cargo test time_stats_rates --all > .omo/evidence/task-4-rates.txt
    Expected: tests pass; zero-duration case returns finite value, not inf/NaN
    Evidence: .omo/evidence/task-4-rates.txt

  Scenario: Duration formatting handles edge cases
    Tool: Bash
    Steps: cargo test time_stats_duration_format --all > .omo/evidence/task-4-duration.txt
    Expected: tests pass for seconds, minutes, and hours formats
    Evidence: .omo/evidence/task-4-duration.txt
  ```

  **Commit**: YES | Message: `feat(time): add deterministic session timing` | Files: [`src/time.rs`, `src/state.rs`, `tests/**`]

- [x] 5. Implement Notification, Notifier, fake notifier, and duplicate dispatcher

  **What to do**: Implement `src/notifier.rs` with `AlertLevel::{Info, Warn, Critical}`, `Notification { event_type: String, level: u8, alert_level, emoji: Option<String>, terminal_text: String, remote_text: String, timestamp: DateTime<Utc>, mention: bool }`, a synchronous Phase 1 trait `Notifier { fn send(&mut self, notification: &Notification) -> anyhow::Result<()>; }`, `FakeNotifier` for tests, and `NotificationDispatcher` that applies log level routing and duplicate suppression. Level semantics: `0 ignore`, `1 terminal`, `2 terminal plus future remote but terminal-only in Phase 1`, `3 terminal plus future remote plus mention but terminal-only in Phase 1`; `mention = level >= 3`. Duplicate suppression uses `[monitor].duplicate_max = 5` by default; after five consecutive identical `event_type` + `terminal_text`, dispatch exactly one level-1 `duplicate_suppression` notification containing `Suppressing further duplicate messages`, then suppress until event type or text changes.
  **Must NOT do**: Do not implement Matrix HTTP. Do not put duplicate suppression in `TerminalNotifier`.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: abstraction must be stable for Phase 2 without overengineering.
  - Skills: [] - No specialized skill required.
  - Omitted: [`notion-api`, `github-cli`] - No external service interaction.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [11,12,13] | Blocked By: [1]

  **References**:
  - Target: `src/notifier.rs` - Notification model, trait, dispatcher, fake notifier.
  - Target: `src/config.rs` - log level config values.
  - User contract: log levels `0-3` from request.

  **Acceptance Criteria**:
  - [ ] `cargo test notifier --all` exits `0`.
  - [ ] Duplicate suppression test sends first `duplicate_max = 5` notifications, then exactly one `duplicate_suppression` notice containing `Suppressing further duplicate messages`, then resumes after event text changes.
  - [ ] Level `0` emits nothing; level `3` sets `mention = true`.

  **QA Scenarios**:
  ```
  Scenario: Duplicate suppression stops spam
    Tool: Bash
    Steps: cargo test notifier_duplicate_suppression --all > .omo/evidence/task-5-duplicates.txt
    Expected: test passes; fake notifier receives N originals plus one suppression notice
    Evidence: .omo/evidence/task-5-duplicates.txt

  Scenario: Level routing is Matrix-ready but terminal-only
    Tool: Bash
    Steps: cargo test notifier_level_routing --all > .omo/evidence/task-5-levels.txt
    Expected: levels 1-3 dispatch to terminal path; level 0 ignored; level 3 mention true
    Evidence: .omo/evidence/task-5-levels.txt
  ```

  **Commit**: YES | Message: `feat(notifier): add notification dispatcher` | Files: [`src/notifier.rs`, `src/config.rs`, `tests/**`]

- [x] 6. Create sanitized fixture policy and minimal Journal fixtures

  **What to do**: Add `tests/fixtures/README.md` stating raw `/home/ubuntu/Elite Dangerous` logs are read-only local inputs and must never be committed. Create minimized sanitized fixtures: `journal_minimal_start.log`, `journal_combat_bounty.log`, `journal_missions.log`, `journal_damage_fighter.log`, `journal_malformed_unknown.log`, `journal_warning_clock.log`. Fixtures must use synthetic commander/system/faction/ship names, no real paths, no real commander names, no carrier names, no private chat text. Include enough fields to test required Phase 1 behavior.
  **Must NOT do**: Do not copy full raw Journal files. Do not include private commander/system/faction names from local logs unless replaced with synthetic values.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: fixture safety and coverage are both important.
  - Skills: [`secret-guard`] - Reason: ensure fixtures do not leak sensitive/private data.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [3,14] | Blocked By: [1]

  **References**:
  - Local read-only samples: `/home/ubuntu/Elite Dangerous/Journal.180729194257.01.log` - Bounty/FighterDestroyed inspiration only.
  - Local read-only samples: `/home/ubuntu/Elite Dangerous/Journal.181214225820.01.log` - ShipTargeted inspiration only.
  - Local read-only samples: `/home/ubuntu/Elite Dangerous/Journal.190422050045.01.log` - MissionRedirected inspiration only.
  - Target: `tests/fixtures/README.md` - fixture privacy policy.

  **Acceptance Criteria**:
  - [ ] `grep -R "/home/ubuntu/Elite Dangerous" tests/fixtures` returns no matches except in `tests/fixtures/README.md` policy text if included.
  - [ ] `cargo test fixture_sanity --all` exits `0` and validates each fixture is line-delimited JSON except the deliberate malformed fixture line.
  - [ ] Fixture README explicitly says raw Journals must not be committed.

  **QA Scenarios**:
  ```
  Scenario: Fixture set is minimal and parseable
    Tool: Bash
    Steps: cargo test fixture_sanity --all > .omo/evidence/task-6-fixtures.txt
    Expected: all sanitized fixtures validate; malformed fixture has exactly one deliberate invalid line
    Evidence: .omo/evidence/task-6-fixtures.txt

  Scenario: Raw local paths are not embedded in fixtures
    Tool: Bash
    Steps: grep -R "/home/ubuntu/Elite Dangerous/Journal" tests/fixtures > .omo/evidence/task-6-raw-paths.txt || true
    Expected: evidence file is empty
    Evidence: .omo/evidence/task-6-raw-paths.txt
  ```

  **Commit**: YES | Message: `test(fixtures): add sanitized journal samples` | Files: [`tests/fixtures/**`, `tests/**`]

- [x] 7. Implement Journal discovery, file selection, and preload boundaries

  **What to do**: Implement `src/journal.rs` discovery for `Journal.*.log`. Default Windows path resolves from `USERPROFILE`; Linux requires explicit `--journal` or `--set-file` unless the Windows path exists. Newest selection sorts by parsed filename timestamp for both legacy `Journal.YYMMDDHHMMSS.01.log` and Odyssey-style `Journal.YYYY-MM-DDTHHMMSS.01.log`, then mtime fallback. Implement `--file-select` listing newest `recent_files` entries with deterministic numeric choices. Implement preload read of selected file that returns parsed events and final byte offset; preload must not dispatch notifications. Implement `--reset-session` hook contract to clear counters after preload.
  **Must NOT do**: Do not implement journal rotation. Do not modify Journal files. Do not require Windows path to exist during Linux tests.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: cross-platform path/discovery edge cases.
  - Skills: [] - No specialized skill required.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [8,9] | Blocked By: [2]

  **References**:
  - Target: `src/journal.rs` - discovery, preload, file select.
  - Target: `src/config.rs` - `journal.folder`, `recent_files`.
  - Local read-only folder: `/home/ubuntu/Elite Dangerous` - optional real discovery validation.

  **Acceptance Criteria**:
  - [ ] `cargo test journal_discovery --all` exits `0`.
  - [ ] Empty temp journal dir returns runtime error mapped to exit code `1`.
  - [ ] Mixed legacy and ISO-style filenames select newest parsed timestamp deterministically.
  - [ ] Preload returns offset at EOF and dispatches zero notifications in tests.

  **QA Scenarios**:
  ```
  Scenario: Newest Journal selected deterministically
    Tool: Bash
    Steps: cargo test journal_discovery_newest_by_filename --all > .omo/evidence/task-7-newest.txt
    Expected: test selects the newest parsed Journal filename, not lexicographic accident
    Evidence: .omo/evidence/task-7-newest.txt

  Scenario: Empty folder maps to exit code 1
    Tool: Bash
    Steps: cargo test journal_discovery_empty_dir_error --all > .omo/evidence/task-7-empty-dir.txt
    Expected: test passes and asserts runtime error kind used by main exit code 1
    Evidence: .omo/evidence/task-7-empty-dir.txt
  ```

  **Commit**: YES | Message: `feat(journal): discover and preload journal files` | Files: [`src/journal.rs`, `src/main.rs`, `tests/**`]

- [x] 8. Implement replay mode using parser, preload semantics, and deterministic output

  **What to do**: Implement `replay` subcommand. `cargo run -- replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` reads from start to EOF, processes every complete line, uses Journal timestamps for state/warnings, prints newline event logs plus final `Session summary`, and exits `0`. Malformed lines warn and continue. `--reset-session` is accepted as a global compatibility flag but has no effect in `replay`; print exactly one warning containing `--reset-session has no effect in replay` and continue.
  **Must NOT do**: Do not sleep according to Journal timestamps. Do not follow EOF in replay. Do not emit crossterm control characters when `--no-status-line` is set.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: integrates CLI, parser, journal reader, monitor pipeline.
  - Skills: [] - No specialized skill required.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [14] | Blocked By: [2,3,7]

  **References**:
  - Target: `src/main.rs` - replay command wiring.
  - Target: `src/journal.rs` - sequential file reading.
  - Target: `tests/fixtures/journal_combat_bounty.log` - deterministic replay fixture.

  **Acceptance Criteria**:
  - [ ] `cargo run -- replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` exits `0`.
  - [ ] Replay output contains `Cargo scan`, `Kill`, and `Session summary`.
  - [ ] Replay malformed fixture exits `0`, prints warning fragment `Malformed journal line`, and continues to summary.
  - [ ] Replay with `--reset-session` exits `0` and prints warning fragment `--reset-session has no effect in replay` exactly once.

  **QA Scenarios**:
  ```
  Scenario: Combat replay produces deterministic output
    Tool: Bash
    Steps: cargo run -- replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line > .omo/evidence/task-8-combat-replay.txt
    Expected: output contains Cargo scan, Kill, Session summary; command exits 0
    Evidence: .omo/evidence/task-8-combat-replay.txt

  Scenario: Malformed replay continues
    Tool: Bash
    Steps: cargo run -- replay --set-file tests/fixtures/journal_malformed_unknown.log --no-status-line > .omo/evidence/task-8-malformed-replay.txt 2>&1
    Expected: output contains Malformed journal line and Session summary; command exits 0
    Evidence: .omo/evidence/task-8-malformed-replay.txt
  ```

  **Commit**: YES | Message: `feat(replay): process journal files deterministically` | Files: [`src/main.rs`, `src/journal.rs`, `src/monitor.rs`, `tests/**`]

- [x] 9. Implement polling live tail for selected/current Journal file

  **What to do**: Implement live `watch` mode tailing the selected latest or explicit file using polling. After preload, seek to EOF and process only appended complete lines. Keep incomplete trailing line buffered until newline. Handle CRLF/LF, UTF-8 losslessly where possible with clear warning on invalid bytes. Gracefully exit `0` on Ctrl-C. On file truncation, warn and reset offset to current EOF. Poll interval defaults to `1000ms` and is configurable via `--poll-interval-ms`.
  **Must NOT do**: Do not use filesystem writes to Journal files except temp files in tests. Do not implement rotation. Do not busy-loop.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: tail correctness has edge cases.
  - Skills: [] - No specialized skill required.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [14] | Blocked By: [2,7]

  **References**:
  - Target: `src/journal.rs` - tail reader and offsets.
  - Target: `src/main.rs` - watch command lifecycle.
  - External: `https://docs.rs/tokio/latest/tokio/fs/index.html` - async file APIs if used.

  **Acceptance Criteria**:
  - [ ] `cargo test live_tail --all` exits `0`.
  - [ ] Test appends partial line then completes it; monitor processes exactly once after newline.
  - [ ] Test proves preloaded last line is not duplicated after live starts.

  **QA Scenarios**:
  ```
  Scenario: Live tail waits for complete line
    Tool: Bash
    Steps: cargo test live_tail_partial_line --all > .omo/evidence/task-9-partial-line.txt
    Expected: test passes; fake notifier receives no event before newline and one event after newline
    Evidence: .omo/evidence/task-9-partial-line.txt

  Scenario: Preload-to-live boundary does not duplicate
    Tool: Bash
    Steps: cargo test live_tail_no_preload_duplicate --all > .omo/evidence/task-9-boundary.txt
    Expected: test passes; appended lines only are processed after EOF offset
    Evidence: .omo/evidence/task-9-boundary.txt
  ```

  **Commit**: YES | Message: `feat(journal): tail live journal updates` | Files: [`src/journal.rs`, `src/main.rs`, `tests/**`]

- [x] 10. Implement SessionState and massacre mission tracking

  **What to do**: Implement `src/state.rs` with `SessionState` fields requested by user plus internal timestamp vectors for recent rates. Handle session start/end rules: start on RES drop, planetary ring `Location`, first `Bounty`/`FactionKillBond`, or first pirate/security-relevant `ShipTargeted`; end on `SupercruiseEntry`, `FSDJump`, `Music` MainMenu, `Shutdown`, `Died`. Track commander, ship, system, mode, shield state, ship hull, fighter hull/alive, cargo scans, kills, bounty total, active massacre mission IDs, mission total/completed. Massacre detection: mission name/localized name contains case-insensitive `Massacre` or known journal massacre token; `MissionRedirected` increments completed when massacre; `MissionCompleted/Failed/Abandoned` removes active ID.
  **Must NOT do**: Do not count mission progress as observed kills. Do not assume every bounty is a mission kill.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: state transitions and ED Journal caveats require careful reasoning.
  - Skills: [] - No specialized skill required.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [11,12] | Blocked By: [3,4]

  **References**:
  - Target: `src/state.rs` - SessionState and transition methods.
  - Target: `tests/fixtures/journal_missions.log` - massacre mission fixture.
  - External: `https://github.com/PsiPab/ED-AFK-Monitor/blob/6d11651f2992d801ebb33cf81a9fd35b01354244/README.md#L103-L117` - Journal caveats, behavior-level only.

  **Acceptance Criteria**:
  - [ ] `cargo test session_state --all` exits `0`.
  - [ ] Tests prove `Bounty` and `FactionKillBond` increment observed kills; mission redirect increments mission completed only.
  - [ ] Tests prove session start/end toggles active session according to locked rules.
  - [ ] Tests prove rates and last-kill/last-scan fields update from event timestamps.

  **QA Scenarios**:
  ```
  Scenario: Mission tracking stays separate from observed kills
    Tool: Bash
    Steps: cargo test session_state_missions_not_kills --all > .omo/evidence/task-10-mission-separation.txt
    Expected: mission_completed increments; kills unchanged unless Bounty/FactionKillBond occurs
    Evidence: .omo/evidence/task-10-mission-separation.txt

  Scenario: Session lifecycle follows AFK boundaries
    Tool: Bash
    Steps: cargo test session_state_start_end_rules --all > .omo/evidence/task-10-lifecycle.txt
    Expected: RES/ring/first kill/pirate scan start session; supercruise/jump/menu/shutdown/died end session
    Evidence: .omo/evidence/task-10-lifecycle.txt
  ```

  **Commit**: YES | Message: `feat(state): track afk session statistics` | Files: [`src/state.rs`, `tests/**`, `tests/fixtures/**`]

- [x] 11. Implement monitor event handling into typed notifications

  **What to do**: Implement `src/monitor.rs` converting `JournalEvent` into state updates and `Notification`s. Event notification coverage: cargo scan from `ShipTargeted`/`ReceiveText` pirate scan evidence; kill from `Bounty`/`FactionKillBond`; mission active/completed from `Missions`, `MissionAccepted`, `MissionRedirected`, `MissionCompleted`, `MissionFailed`, `MissionAbandoned`; shields from `ShieldState`; hull/fighter from `HullDamage`, `FighterDestroyed`, `LaunchFighter`; cargo loss from `EjectCargo`; fuel from `ReservoirReplenished`/fuel fields if present; security warnings from hostile/security `ShipTargeted`/`ReceiveText`; session start/end from lifecycle events; death from `Died`. Use config `log_levels` keys exactly as requested. Notification text may be independently worded but must include stable test fragments like `Cargo scan`, `Kill`, `Shield down`, `Fighter destroyed`, `Ship destroyed`.
  **Must NOT do**: Do not hardwire terminal output inside monitor. Do not copy upstream messages.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: central feature mapping and clean abstraction.
  - Skills: [] - No specialized skill required.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [12,13,14] | Blocked By: [3,5,10]

  **References**:
  - Target: `src/monitor.rs` - event-to-state/notification logic.
  - Target: `src/notifier.rs` - notification dispatch.
  - Target: `src/config.rs` - log level keys.
  - External: `https://github.com/PsiPab/ED-AFK-Monitor/blob/6d11651f2992d801ebb33cf81a9fd35b01354244/README.md#L29-L50` - behavior-level feature scope only.

  **Acceptance Criteria**:
  - [ ] `cargo test monitor_events --all` exits `0`.
  - [ ] Fake notifier tests cover cargo scan, kill, mission redirect, shield down/restored, fighter destroyed, hull damage, died, cargo lost.
  - [ ] Level `0` events update state if needed but do not emit notification.

  **QA Scenarios**:
  ```
  Scenario: Combat fixture emits core notifications
    Tool: Bash
    Steps: cargo test monitor_events_combat_notifications --all > .omo/evidence/task-11-combat-events.txt
    Expected: fake notifier receives Cargo scan and Kill notifications with correct levels
    Evidence: .omo/evidence/task-11-combat-events.txt

  Scenario: Danger fixture emits critical notifications
    Tool: Bash
    Steps: cargo test monitor_events_damage_notifications --all > .omo/evidence/task-11-danger-events.txt
    Expected: fake notifier receives Shield down, Fighter destroyed, hull/died notifications; level 3 mention true where configured
    Evidence: .omo/evidence/task-11-danger-events.txt
  ```

  **Commit**: YES | Message: `feat(monitor): map journal events to notifications` | Files: [`src/monitor.rs`, `src/state.rs`, `src/notifier.rs`, `tests/**`]

- [x] 12. Implement no-kill and low-kill-rate warning scheduler

  **What to do**: Implement warning checks in monitor using injected clock and state timestamps. Warnings activate only when session is active and not preload. Initial no-kill warning fires once after `warn_no_kills_initial_minutes` if no observed kill since session start. Later no-kill warning fires after `warn_no_kills_minutes` since last kill and respects `warn_cooldown_minutes`. Low-kill-rate warning fires when total observed kill rate is below `warn_kill_rate` after at least initial no-kill window has elapsed and respects cooldown. A new kill resets no-kill timer and low-rate cooldown eligibility. Replay mode evaluates against Journal timestamps and produces deterministic warning positions.
  **Must NOT do**: Do not warn before session start. Do not warn during preload. Do not base tests on real wall-clock sleeping.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: timing edge cases require deterministic tests.
  - Skills: [] - No specialized skill required.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [13,14] | Blocked By: [4,5,10,11]

  **References**:
  - Target: `src/monitor.rs` - warning scheduler.
  - Target: `src/time.rs` - fake clock/time source.
  - Target: `tests/fixtures/journal_warning_clock.log` - deterministic warning fixture.

  **Acceptance Criteria**:
  - [ ] `cargo test warnings --all` exits `0`.
  - [ ] Tests prove no warnings during preload.
  - [ ] Tests prove no-kill initial/later thresholds and cooldown.
  - [ ] Tests prove low kill rate warning resets after kill and respects cooldown.

  **QA Scenarios**:
  ```
  Scenario: No-kill warning fires once after threshold
    Tool: Bash
    Steps: cargo test warnings_no_kill_threshold --all > .omo/evidence/task-12-no-kill.txt
    Expected: fake notifier receives exactly one no_kills warning after configured threshold
    Evidence: .omo/evidence/task-12-no-kill.txt

  Scenario: Warnings are suppressed during preload
    Tool: Bash
    Steps: cargo test warnings_disabled_during_preload --all > .omo/evidence/task-12-preload.txt
    Expected: preload processing advances timestamps but fake notifier receives zero warning notifications
    Evidence: .omo/evidence/task-12-preload.txt
  ```

  **Commit**: YES | Message: `feat(monitor): warn on idle and low kill rate` | Files: [`src/monitor.rs`, `src/time.rs`, `tests/**`]

- [x] 13. Implement terminal notifier and live status rendering

  **What to do**: Implement `src/terminal.rs` with `TerminalNotifier` for event logs and a `render_status_line(&SessionState, &Config, now) -> String` function. TTY mode uses crossterm clear-current-line/repaint/color if available; non-TTY or `--no-status-line` prints newline logs and no control characters. Status format must contain stable fragments: `Kills {n}`, `{rate}/h`, `Scans {n}`, `Last kill {duration|--}`, `Missions {completed}/{total}`, `Shield OK|DOWN|?`. Event log format must include local or UTC time based on config and independently worded emoji/text fragments.
  **Must NOT do**: Do not add full TUI. Do not make tests depend on terminal dimensions unless mocked.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - Reason: terminal UX requires clear output and non-TTY safety.
  - Skills: [] - No browser/UI skill needed; this is terminal UI only.
  - Omitted: [`visual-qa`] - Final visual QA covers terminal behavior; task-level tests are string/control-code checks.

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: [14] | Blocked By: [5,11,12]

  **References**:
  - Target: `src/terminal.rs` - event and status rendering.
  - Target: `src/text.rs` - truncation/format helpers.
  - External: `https://docs.rs/crossterm/latest/crossterm/` - terminal control.

  **Acceptance Criteria**:
  - [ ] `cargo test terminal_rendering --all` exits `0`.
  - [ ] Non-TTY test output contains no ANSI clear-line/control sequences.
  - [ ] Status render test contains `Kills 71`, `22.4/h`, `Scans 96`, `Last kill 58s`, `Missions 5/20`, `Shield OK` for fixed state.

  **QA Scenarios**:
  ```
  Scenario: Fixed status renders expected fragments
    Tool: Bash
    Steps: cargo test terminal_status_fixed_fragments --all > .omo/evidence/task-13-status.txt
    Expected: test passes with exact fixed fragments for kills/rates/scans/mission/shield
    Evidence: .omo/evidence/task-13-status.txt

  Scenario: Non-TTY output has no control characters
    Tool: Bash
    Steps: cargo test terminal_non_tty_plain_output --all > .omo/evidence/task-13-non-tty.txt
    Expected: test passes; output string contains no crossterm clear-line escape sequences
    Evidence: .omo/evidence/task-13-non-tty.txt
  ```

  **Commit**: YES | Message: `feat(terminal): render events and live status` | Files: [`src/terminal.rs`, `src/text.rs`, `src/main.rs`, `tests/**`]

- [x] 14. Add end-to-end replay, live simulation, and optional real Journal regression tests

  **What to do**: Add integration tests using `assert_cmd` for CLI replay and temp-file live simulation. Add ignored test `tests/real_journal_replay.rs` that reads `/home/ubuntu/Elite Dangerous` read-only if present, scans known sample files for at least one Bounty, ShipTargeted, MissionRedirected, ShieldState, FighterDestroyed, and HullDamage event, and runs parser/state pipeline without committing any raw content. If folder is absent, ignored test prints skip and returns success. Ensure CI runs only non-ignored sanitized fixture tests.
  **Must NOT do**: Do not write to `/home/ubuntu/Elite Dangerous`. Do not include raw Journal output in committed snapshots. Do not require ignored test in CI.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: end-to-end tests must be deterministic and privacy-safe.
  - Skills: [`secret-guard`] - Reason: verify no raw secrets/private Journal data in fixtures/evidence committed.
  - Omitted: [] - None.

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: [16] | Blocked By: [8,9,11,12,13]

  **References**:
  - Target: `tests/cli_replay.rs` - CLI replay integration.
  - Target: `tests/live_tail.rs` - temp-file live simulation.
  - Target: `tests/real_journal_replay.rs` - ignored local regression.
  - Local read-only samples: `/home/ubuntu/Elite Dangerous/Journal.180729194257.01.log`, `/home/ubuntu/Elite Dangerous/Journal.181214225820.01.log`, `/home/ubuntu/Elite Dangerous/Journal.190422050045.01.log`, `/home/ubuntu/Elite Dangerous/Journal.170814020512.01.log`, `/home/ubuntu/Elite Dangerous/Journal.180725114837.01.log`.

  **Acceptance Criteria**:
  - [ ] `cargo test --all` exits `0` without needing `/home/ubuntu/Elite Dangerous`.
  - [ ] `cargo test --test real_journal_replay -- --ignored` exits `0` in current dev environment.
  - [ ] `cargo run -- replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` exits `0` and contains `Cargo scan`, `Kill`, `Session summary`.

  **QA Scenarios**:
  ```
  Scenario: Sanitized replay is end-to-end deterministic
    Tool: Bash
    Steps: cargo run -- replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line > .omo/evidence/task-14-sanitized-replay.txt
    Expected: output includes Cargo scan, Kill, Session summary; no ANSI clear-line sequence
    Evidence: .omo/evidence/task-14-sanitized-replay.txt

  Scenario: Real local Journal replay is read-only and optional
    Tool: Bash
    Steps: cargo test --test real_journal_replay -- --ignored > .omo/evidence/task-14-real-journal.txt
    Expected: command exits 0; output reports parsed known event categories or skip if folder missing
    Evidence: .omo/evidence/task-14-real-journal.txt
  ```

  **Commit**: YES | Message: `test(replay): cover cli and real journal regression` | Files: [`tests/**`, `tests/fixtures/**`]

- [x] 15. Write README and config documentation for Phase 1 only

  **What to do**: Write `README.md` and finalize `config.example.toml`. README must describe the project as independently implemented and inspired by AFK monitoring use cases / Elite Journal semantics, not as a fork or port. Include Windows default path, Linux test examples, CLI usage, replay usage, config/log-level semantics, TDD/fixture privacy policy, Phase 2 Matrix roadmap, and explicit non-goals. Include copy-pastable commands: `cargo run -- watch --journal "/path/to/Elite Dangerous"`, `cargo run -- replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line`, `cargo test --all`, `cargo test --test real_journal_replay -- --ignored`.
  **Must NOT do**: Do not copy upstream README wording. Do not document Matrix config as usable in Phase 1. Do not include access tokens/secrets examples.

  **Recommended Agent Profile**:
  - Category: `writing` - Reason: concise technical documentation and guardrails.
  - Skills: [] - No specialized skill required.
  - Omitted: [`secret-guard`] - No secrets should be present; final verification will scan.

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: [16] | Blocked By: [2,7,8,13]

  **References**:
  - Target: `README.md` - user docs.
  - Target: `config.example.toml` - config reference.
  - External: `https://github.com/PsiPab/ED-AFK-Monitor/blob/6d11651f2992d801ebb33cf81a9fd35b01354244/README.md#L29-L50` - behavior-level feature inspiration; do not copy text.

  **Acceptance Criteria**:
  - [ ] `grep -n "Discord\|WebUI\|EDMC\|auto relog\|Matrix command" README.md` only appears under non-goals or deferred roadmap.
  - [ ] README includes exact artifact names `ed-afk-monitor-x86_64-unknown-linux-gnu.tar.gz` and `ed-afk-monitor-x86_64-pc-windows-msvc.zip`.
  - [ ] README includes the raw Journal privacy warning.

  **QA Scenarios**:
  ```
  Scenario: README has required commands
    Tool: Bash
    Steps: grep -E "cargo run -- (watch|replay)|cargo test --all|real_journal_replay" README.md > .omo/evidence/task-15-readme-commands.txt
    Expected: evidence contains watch, replay, cargo test --all, and ignored real replay command
    Evidence: .omo/evidence/task-15-readme-commands.txt

  Scenario: Phase 2 is documented as deferred
    Tool: Bash
    Steps: grep -n "Matrix" README.md > .omo/evidence/task-15-matrix-roadmap.txt
    Expected: Matrix appears only as deferred Phase 2/roadmap, not active Phase 1 configuration
    Evidence: .omo/evidence/task-15-matrix-roadmap.txt
  ```

  **Commit**: YES | Message: `docs(readme): document phase one usage` | Files: [`README.md`, `config.example.toml`, `tests/fixtures/README.md`]

- [x] 16. Add GitHub Actions CI and release artifact workflows

  **What to do**: Add `.github/workflows/ci.yml` running on push/PR for `ubuntu-latest` and `windows-latest`: checkout, install stable Rust, cache cargo, run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all`. Add `.github/workflows/release.yml` triggered by tags `v*`: build `--release` on native `ubuntu-latest` and `windows-latest`; package Linux binary as `ed-afk-monitor-x86_64-unknown-linux-gnu.tar.gz`; package Windows `.exe` as `ed-afk-monitor-x86_64-pc-windows-msvc.zip`; upload artifacts. Add tests or script checks to validate workflow YAML contains both runners and exact artifact names.
  **Must NOT do**: Do not require real Journal files in CI. Do not cross-compile Windows from Linux in Phase 1; use native Windows runner. Do not skip clippy warnings.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: CI/release must be cross-platform and exact.
  - Skills: [] - No GitHub write/PR creation required.
  - Omitted: [`github-cli`] - Workflow files are local project files; no remote inspection needed.

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: [] | Blocked By: [1,14,15]

  **References**:
  - Target: `.github/workflows/ci.yml` - quality gate CI.
  - Target: `.github/workflows/release.yml` - tag release artifacts.
  - External: `https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions` - workflow syntax.

  **Acceptance Criteria**:
  - [ ] Workflow YAML contains `ubuntu-latest` and `windows-latest`.
  - [ ] Workflow YAML contains exact artifact names `ed-afk-monitor-x86_64-unknown-linux-gnu.tar.gz` and `ed-afk-monitor-x86_64-pc-windows-msvc.zip`.
  - [ ] Workflow YAML does not contain `real_journal_replay -- --ignored` or any `/home/ubuntu/Elite Dangerous` dependency.
  - [ ] Local gates pass: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all`.

  **QA Scenarios**:
  ```
  Scenario: Local CI gates pass before relying on Actions
    Tool: Bash
    Steps: sh -c 'cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all' > .omo/evidence/task-16-local-gates.txt 2>&1
    Expected: all commands exit 0
    Evidence: .omo/evidence/task-16-local-gates.txt

  Scenario: Workflow artifact names are exact
    Tool: Bash
    Steps: grep -R "ed-afk-monitor-x86_64-unknown-linux-gnu.tar.gz\|ed-afk-monitor-x86_64-pc-windows-msvc.zip" .github/workflows > .omo/evidence/task-16-artifacts.txt
    Expected: evidence contains both exact artifact names
    Evidence: .omo/evidence/task-16-artifacts.txt

  Scenario: CI does not run ignored real Journal replay
    Tool: Bash
    Steps: grep -R "real_journal_replay\|/home/ubuntu/Elite Dangerous" .github/workflows > .omo/evidence/task-16-no-real-journal-ci.txt || true
    Expected: evidence file is empty
    Evidence: .omo/evidence/task-16-no-real-journal-ci.txt
  ```

  **Commit**: YES | Message: `ci(release): build linux and windows artifacts` | Files: [`.github/workflows/ci.yml`, `.github/workflows/release.yml`, `README.md`]

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.
- [x] F1. Plan Compliance Audit — oracle
  - Verify every TODO acceptance criterion was run or explicitly impossible with evidence.
  - Verify clean-room guardrails were followed and upstream code/text was not copied.
  - Verify no Matrix/Discord/WebUI/automation/database scope slipped into Phase 1.
- [x] F2. Code Quality Review — unspecified-high
  - Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all`.
  - Inspect module boundaries for excessive coupling or AI slop.
- [x] F3. Hands-on Runtime QA — unspecified-high
  - Execute `cargo run -- replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line`.
  - Execute `cargo test --test real_journal_replay -- --ignored` in the current Linux environment.
  - For terminal behavior, capture non-TTY output and confirm no control characters.
- [x] F4. Scope Fidelity Check — deep
  - Verify requested events/AFK monitoring items are covered by parser/state/monitor tests or explicitly deferred with reason.
  - Verify Windows/Linux cross-platform assumptions are covered by tests or CI workflow.

## Commit Strategy
- Commit after each task using the task-specific message.
- Do not push unless explicitly requested.
- Before any commit containing fixtures/docs/config, run a secrets/privacy scan over staged files; raw `/home/ubuntu/Elite Dangerous` logs must never be staged.
- If hooks modify files, follow repository git safety rules; do not amend unless allowed by the Git Safety Protocol.

## Success Criteria
- `ed-afk-monitor` can replay sanitized fixtures and emit deterministic terminal event/status output.
- `ed-afk-monitor watch --journal /home/ubuntu/Elite Dangerous --no-status-line` can preload and tail the selected latest Journal without modifying the folder.
- Parser covers all listed Phase 1 event names and safely handles unknown/malformed lines.
- SessionState tracks kills, scans, bounties, mission active/completed, shields, hull/fighter state, system/ship/commander/mode, last kill/scan, and rates.
- No-kill and low-kill-rate warnings are deterministic and cooldown-safe.
- Notification/Notifier abstraction exists and is Matrix-ready, but Matrix is not implemented.
- CI workflows define Linux and Windows quality gates and tag release artifacts with exact requested names.
