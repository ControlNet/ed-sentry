# AFK watcher abstraction notes

Date: 2026-06-26

Context: Todo 6 of `.omo/plans/afk-checklist-watcher.md` added the runtime watcher adapter without integrating terminal or desktop watch loops.

Findings:

- `src/app/runtime/file_watcher.rs` is the public runtime seam for later loop integration. `AfkFileWatcherStart::start(selected_file)` returns either `Watching { watcher, events }` or `PollingFallback { warning }`.
- `AfkFileWatcher` owns and retains `notify::RecommendedWatcher`; dropping it stops the OS watcher through the notify guard.
- The production watcher registers the selected file parent directory with `RecursiveMode::NonRecursive` and dispatches through a bounded `tokio::sync::mpsc` channel.
- `WatchedFileSet` filters normalized paths to the exact selected file, `Status.json`, and `Cargo.json`. It intentionally ignores unselected `Journal.*.log` files and does not implement Journal auto-rotation.
- `DebouncedWatcherEvents` and `CompanionReadRetry` are pure deterministic helpers so Todo 7/8 can coalesce duplicate companion events and retry likely partial writes without adding real-time sleeps to unit tests.
- Required evidence is saved in `.omo/evidence/afk-checklist-watcher/task-6-watcher-tests.txt` and `.omo/evidence/afk-checklist-watcher/task-6-filter-retry.txt`.

Docs update, 2026-06-26:

- README now records the final user-visible behavior: selected Journal, `Status.json`, and `Cargo.json` watcher events drive low-latency updates while `poll_interval_ms` remains fallback and housekeeping.
- README describes the `Checklist` panel's `PASS`/`FAIL`/`UNKNOWN` values for `Hardpoints deployed`, `Engine pips zero`, and `Cargo loaded`.
- Cargo readiness is documented as non-empty ship cargo from `Cargo.json`, not market value.
