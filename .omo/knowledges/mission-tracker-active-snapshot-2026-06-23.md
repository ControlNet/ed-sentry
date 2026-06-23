Mission Tracker must treat the Elite Dangerous `Missions` journal event as a source of visible active missions, not only as a retention filter.

Root cause fixed on 2026-06-23: `MissionTracker` used `Missions.Active` only to remove stale active missions. When a commander already had accepted missions before the currently selected Journal file emitted `MissionAccepted`, the WebUI Mission Tracker could show blank even though the game had active missions.

Required behavior:

- When `Missions.Active` contains a `MissionID` that is not already tracked, create a placeholder active mission from the snapshot entry.
- Preserve positive snapshot `Expires` values as absolute mission expiry timestamps based on the `Missions` event timestamp.
- Preserve existing tracked missions when their `MissionID` appears in `Missions.Active`.
- Continue removing active/redirected missions that are missing from a present `Missions.Active` snapshot.
- Use the snapshot entry name to classify obvious mission kinds such as delivery/collect/mining/altruism as trade and massacre as massacre, even when detailed progress fields are unavailable until later events.

Verification signals used for this fix:

- `cargo test --test mission_tracker mission_tracker_creates_active_missions_from_snapshot_without_accept_event` failed before implementation because mission `7001001` was absent.
- After implementation, `/api/snapshot` from a live WebUI runtime using `tests/fixtures/journal_missions.log` returned `mission_ids=[7001001, 7001002]`, `active_count=1`, and `7001001` as `state=active`, `kind=trade`, with expiry preserved from the snapshot.

Follow-up learned from OD Elite Tracker comparison:

- OD Elite Tracker does not derive Massacre target counts from `Missions.Active`; its `MassacreMissionStore` creates massacre missions from `MissionAccepted` only when `KillCount`, `TargetFaction`, and pirate `TargetType` are present.
- This project must preload recent journal history into `MissionTracker` before processing the selected/current journal so active snapshot rows can be joined back to the earlier `MissionAccepted` details. Do not fake Massacre `0/0` as a complete progress value when the earlier accepted event was simply outside the selected journal file.
