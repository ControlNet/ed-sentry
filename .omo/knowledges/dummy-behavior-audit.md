# Dummy behavior audit

Audit focus after the watch no-op bug: CLI/config features must either be wired to runtime behavior or be explicitly documented as deferred/out of scope.

Fixed during the audit:

- `watch` now preloads the selected Journal file, emits matching existing events, then tails appended complete lines until stopped.
- `--file-select` now lists recent Journal files and reads the chosen number from stdin.
- `--debug` now emits runtime diagnostics for selected/preloaded Journal work.
- `live_status` now has a concrete TTY status-line path through `TerminalNotifier::supports_status_line()` and `render_status_line()`.

Resolved follow-up for the remaining parsed config fields:

- `monitor.pirate_names` controls whether scan lines show the target pilot name or the generic `contact` label.
- `monitor.bounty_faction` controls whether kill lines include victim faction text.
- `monitor.bounty_value` controls whether kill lines include credit values.
- `monitor.extended_stats` controls supported event counters such as kill sequence numbers; replay summaries are controlled by the `summary_*` log levels below.
- `log_levels.summary_kills` controls the replay `Kills` summary fragment.
- `log_levels.summary_faction` controls the replay per-victim-faction summary fragment.
- `log_levels.summary_scans` controls the replay `Scans` summary fragment.
- `log_levels.summary_bounties` controls the replay `Bounties` summary fragment.

Direct CLI smoke for the config-driven output used a temporary config with names/factions/values/extended stats disabled and summary kills only; expected output showed upstream-style scan/kill lines such as `Scan: contact`, `Kill: viper`, `Kill: Bond`, and a `Total Stats` kills fragment only.

Remote delivery markers (`Notification.remote_text`, `Notification.mention`, log level 2/3 remote semantics) are intentionally Phase 2 scaffolding and documented as deferred.
