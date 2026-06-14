# AFK risk event ingest

- AFK-relevant risk events should be promoted from raw-preserving known events when their fields are useful for later monitor decisions, but promotion alone must not add default terminal output.
- Promoted on 2026-06-14: `Interdicted`, `Interdiction`, `EscapeInterdiction`, `UnderAttack`, `HeatWarning`, `HeatDamage`, `CockpitBreached`, `SystemsShutdown`, `RebootRepair`, and `SelfDestruct`.
- `Interdicted`, `Interdiction`, `EscapeInterdiction`, and `UnderAttack` expose selected Journal fields for future AFK monitor logic. The remaining risk events currently use `BasicJournalEvent` to preserve typed identity without inventing unused schemas.
- `SessionState` explicitly ignores these promoted events so broad ingest expansion does not make replay/watch noisy by default.
- Keep replay low-noise regression coverage when promoting future Journal events: events can enter the parser without appearing in stdout unless the AFK monitor scenario justifies a notification.
