# ed-afk-dashboard

`ed-afk-dashboard` is an independent Rust CLI for Elite Dangerous Journal AFK monitoring use cases. It reads local Journal files, tracks Phase 1 combat and session signals, and prints terminal output for live watch and replay runs.

This project is not a fork, port, or copy of another monitor. The implementation, code structure, messages, and docs are written for this repository while following Elite Dangerous Journal semantics and the Phase 1 plan.

## Phase 1 Scope

Phase 1 is local CLI monitoring only.

Supported now:

- Find the newest `Journal.*.log` file from a configured folder.
- Watch a selected Journal file, print matching events already present in that file, then tail appended complete lines.
- Replay one sanitized or local Journal file from start to end.
- Track cargo scans, observed kills, bounties, massacre mission progress, shield and hull state, fighter events, fuel reports, cargo loss, death, and session summaries.
- Render terminal event logs and a live status line when the output is a TTY.

Out of scope for Phase 1:

- Matrix delivery and Matrix command handling. Phase 2 Matrix is deferred and has no usable Phase 1 configuration.
- Discord delivery.
- WebUI dashboards.
- EDMC plugin support.
- auto relog, key simulation, game automation, and relog scripting.
- Database storage or historical dashboards.

## Journal Paths

On Windows, the default Journal folder is resolved from the system Saved Games known folder, so it follows Windows folder relocation settings:

```text
<Saved Games>\Frontier Developments\Elite Dangerous
```

On Linux and development machines, pass the folder or a file explicitly:

```bash
cargo run -- --journal "/path/to/Elite Dangerous"
cargo run -- --journal "/home/ubuntu/Elite Dangerous"
cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
```

By default, `ed-afk-dashboard` runs in watch mode and accepts `--journal <folder>` or `--set-file <file>`. Passing `--replay` switches to replay mode; replay requires `--set-file <file>` and rejects `--journal` in Phase 1.

## CLI Usage

Run from the repository root while developing:

```bash
cargo run -- --journal "/path/to/Elite Dangerous"
```

Replay the deterministic sanitized combat fixture:

```bash
cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
```

Run the normal test suite:

```bash
cargo test --all
```

Run the optional ignored real Journal regression test:

```bash
cargo test --test real_journal_replay -- --ignored
```

Expected signals:

- The replay fixture exits `0` and prints reference-style terminal fragments such as `Scan`, `Kill`, and `Total Stats`.
- `cargo test --all` exits `0` without requiring private Journal files.
- The ignored real Journal test exits `0` when local Journals exist. If that test is unavailable in the current checkout, Task 14 hasn't added it yet.

Common flags:

- `--config <file>` loads a TOML config file.
- `--journal <folder>` sets the Journal folder for `watch`.
- `--set-file <file>` selects one Journal file.
- `--file-select` lists recent Journal files and reads the selected number from stdin.
- `--reset-session` clears watch counters after startup preload output. In replay it is accepted for compatibility and prints one no-effect warning.
- `--debug` prints runtime diagnostics such as selected Journal file and preload offsets.
- `--no-status-line` disables the live status line and keeps output newline-safe.
- `--poll-interval-ms <ms>` changes the live polling interval in the default watch mode. `--replay` rejects this flag because it has no replay effect.
- `--replay` reads the selected Journal file from start to finish and exits.

No subcommands are used. If `--replay` is absent, the CLI runs in watch mode.

## Configuration

Use `config.example.toml` as the Phase 1 reference. It contains the locked defaults for `[journal]`, `[monitor]`, and `[log_levels]`.

Config precedence is:

1. CLI flags.
2. Values from `--config <file>`.
3. Built-in defaults.

Missing keys keep their defaults. Wrong typed keys print a warning and keep the default for that key. Malformed TOML exits with code `1`.

Important monitor defaults:

- `live_status = true` enables the TTY status line unless `--no-status-line` is passed.
- `use_utc = false` prints local time by default.
- `poll_interval_ms = 1000` is the default watch polling interval.
- `warn_kill_rate = 20`, `warn_no_kills_initial_minutes = 5`, `warn_no_kills_minutes = 20`, and `warn_cooldown_minutes = 30` control idle and low-rate warnings.
- `duplicate_max = 5` is retained for future remote delivery controls; Phase 1 terminal output does not suppress duplicate notifications so it stays aligned with the upstream console stream.
- `pirate_names = false`, `bounty_faction = false`, and `bounty_value = false` keep default cargo-scan and kill lines concise; set them to `true` to include pilot names, victim factions, and credit values.
- `extended_stats = false` keeps default event lines concise; set it to `true` to include supported event counters such as kill sequence numbers.

Replay summary log levels control individual summary fragments:

- `summary_kills` controls the `Kills` fragment.
- `summary_scans` controls the `Scans` fragment.
- `summary_bounties` controls the `Bounties` fragment.
- `summary_faction` controls per-victim-faction kill totals.
- `summary_merits` controls the Powerplay merits fragment.

Log level values are terminal routing levels in Phase 1:

- `0` ignores that notification type.
- `1` prints to the terminal.
- `2` prints to the terminal and marks the event as future remote-capable.
- `3` prints to the terminal and marks the event as future mention-capable.

Remote delivery is not active in Phase 1. There are no Matrix homeserver, token, room, or command settings to configure.

## Privacy And Fixtures

Raw local Journals are read-only inputs and must not be committed. This includes files under `/home/ubuntu/Elite Dangerous` and any personal Journal folder.

The committed files under `tests/fixtures/` are synthetic and sanitized. They use fake commander, system, faction, ship, mission, and message values. Keep raw commander names, carrier names, chat text, local paths, tokens, credentials, and private log content out of fixtures, docs, and evidence files.

See `tests/fixtures/README.md` for the fixture policy.

## Release Artifacts

The tag release workflow publishes these Phase 1 artifact names:

- `ed-afk-dashboard-x86_64-unknown-linux-gnu.tar.gz`
- `ed-afk-dashboard-x86_64-pc-windows-msvc.zip`

CI runs the normal sanitized test suite on Linux and Windows. Optional ignored real Journal regression tests remain local-only and are not part of CI or release workflows.

## Phase 2 Roadmap

Matrix is deferred to Phase 2. Phase 1 keeps notifier abstractions ready for future delivery, but Matrix is not active, not configured, and not required for any current command.

Future Matrix work may add remote delivery after the local CLI is stable. Matrix command handling remains a non-goal until a later plan explicitly adds it.
