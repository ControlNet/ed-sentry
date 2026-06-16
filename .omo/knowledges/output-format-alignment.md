## Terminal output format alignment

- CLI startup output now uses a reference-style three-line banner: `ed-sentry v260421 by CMDR ControlNet`, followed by `Journal folder`, basename-only `Journal file`, `Commander name`, `Config profile: Default`, `Starting... (Press Ctrl+C to stop)`, and the terminal-only Discord info line.
- Replay and watch both route Journal events through `EventMonitor`, so tests should assert shared notification semantics rather than a replay-only renderer.
- Default visible output tracks the upstream example config: `pirate_names = false`, `bounty_faction = false`, `bounty_value = false`, `extended_stats = false`, `warn_kill_rate = 20`, `warn_no_kills_minutes = 20`, with hard scan/kill log levels at `2`.
- Event output now uses bracketed short timestamps plus emoji, for example `[10:02:00]🔎 Scan: Viper Mk III (Competent)`, `[10:03:00]💥 Kill: Viper Mk III`, `[10:03:05]💥 Kill: Bond (+5s)`, and multi-line `[10:05:00]📝 Total Stats ...` summaries.
- `ShipTargeted` state relevance treats completed targets with a `Ship` field as scan-relevant; monitor notification delivery still requires known target ship IDs and min scan level checks.
- `TerminalNotifier` preserves `\n` in notification text for multi-line summaries while stripping `\r` and other control characters.
- Non-summary `EventMonitor` notifications and CLI startup/path/debug/error output sanitize untrusted Journal/path/config text with `line_safe`, so user data cannot inject extra terminal lines or CSI controls.
- Manual QA on 2026-06-10 used tmux with `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line` and observed banner/startup, scan, kills, cargo loss, fuel report, total stats, and monitor stopped output end-to-end. `target/debug/ed-sentry --help` rendered the expected flat no-subcommand options, and a missing replay file printed the banner plus an open-file error.
