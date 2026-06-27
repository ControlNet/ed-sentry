# Desktop Watch Loop Watcher Integration

- `DesktopRuntime::start` must preserve startup ordering: `MonitorRuntime::start`, silent WebUI startup, delivery setup/status, startup header, startup processing, then watcher-backed monitor task spawn.
- The desktop watcher starts from `runtime.startup().journal_file`, matching the selected Journal path chosen by `MonitorRuntime::start`.
- Desktop watcher events should reuse `watch_runner::watcher_event_cycle`; selected Journal events continue through `LiveTail::poll(parse_journal_line)`, and `Status.json`/`Cargo.json` events continue through `MonitorRuntime::process_companion_update`.
- Keep runtime file IO and delivery separated: compute the `WatchCycle` inside a short `MonitorRuntime` mutex scope, drop that guard, then acquire the delivery mutex and await `deliver_watch_cycle`.
- Fallback behavior is `DesktopFileWatcher { _watcher: None, events: None }` plus the same interval polling path, with the sanitized startup warning emitted once by `DesktopFileWatcher::start`.
