# Broad Journal event support

- Unsupported Elite Dangerous Journal events are now categorized into raw-preserving `JournalEvent` variants instead of collapsing all recognized-but-unmodeled events into `Unknown`.
- `RawJournalEvent` stores `timestamp`, `event`, and the full `serde_json::Value` payload for broad categories such as startup snapshots, station, exploration, navigation, cargo/material, ship/module, mission detail, combat detail, Odyssey, social, Powerplay, squadron, carrier, and colonisation.
- `JournalEvent::raw_payload()` exposes preserved JSON for raw category variants and truly unknown future events.
- Existing Phase 1 typed variants and monitor/state behavior remain unchanged; broad events are ignored by `SessionState` unless promoted to typed behavior later.
- Parser tests iterate every broad event-name table and assert the category variant plus nested raw payload preservation.
