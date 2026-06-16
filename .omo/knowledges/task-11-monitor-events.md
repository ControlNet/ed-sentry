# Task 11 Monitor Event Handling

- `ed_sentry::monitor::EventMonitor<N>` owns Task 11 event processing. Construct it with `EventMonitor::new(notifier, MonitorConfig, LogLevelConfig)` or `EventMonitor::from_runtime_config(notifier, &RuntimeConfig)`.
- `EventMonitor::process_event(&JournalEvent)` clones the previous state, applies `SessionState::apply_event`, then builds at most one typed `Notification` for the triggering event and sends it through `NotificationDispatcher`.
- Level `0` log-level suppression is intentionally delegated to `NotificationDispatcher`; the monitor still updates `SessionState` before dispatch, so muted events can still affect counters and lifecycle.
- Scan notifications are emitted when state scan count increases from `ShipTargeted` or pirate/raider ReceiveText scan evidence. `ShipTargeted` uses `scan_hard`, `security_scan`, or `scan_easy`; ReceiveText scan evidence uses `scan_incoming`.
- Kill notifications use `kill_easy` for `Bounty` and `kill_hard` for `FactionKillBond`. Mission lifecycle notifications use `missions`, while `Missions` snapshots use `missions_all`.
- Danger notifications use `ship_shields`, `ship_hull`, `fighter_hull`, `fighter_down`, `cargo_lost`, and `died`; fuel reports use `fuel_report`, `fuel_low`, or `fuel_critical` based on `FuelMain`.
- Rank-up related notifications use `rank_promotion` and cover Journal `Promotion`, `PowerplayRank`, and `SquadronPromotion`; the default is level `2` so Matrix mentions are emitted when configured.
- Monitor tests live in `tests/monitor_events.rs`; required evidence files are `.omo/evidence/task-11-combat-events.txt` and `.omo/evidence/task-11-danger-events.txt`.
