# Task 8 Replay Mode

- Replay command behavior lives in `src/main.rs`; it selects the runtime `--set-file` through `select_configured_journal_file` and reads the file once with `preload_journal_file`.
- Each complete line is parsed with `parse_journal_line`; valid events update first/last Journal timestamp tracking, and malformed parse records print a warning containing `Malformed journal line` without stopping replay.
- Replay uses the same `EventMonitor` notification path as watch mode. Completed known `ShipTargeted` events print upstream-style `Scan: ...`; `Bounty` and `FactionKillBond` print `Kill` lines.
- Replay output is terminal-safe, does not sleep by Journal timestamps, does not follow EOF, and with `--no-status-line` emits no crossterm status-line controls.
- The final replay monitor output includes `Total Stats` when summary levels are enabled, plus `Monitor stopped (<journal basename>)`.
- Focused tests are in `tests/replay.rs`; use `cargo test --test replay` for replay fragments, malformed continuation, summary log-level controls, and exactly-one reset-session warning.
