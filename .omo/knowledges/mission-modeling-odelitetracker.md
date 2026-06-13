# ODEliteTracker-inspired mission modeling

- Mission modeling is intentionally separate from AFK combat `SessionState`. Use `ed_afk_dashboard::mission::MissionTracker` for dashboard-style mission state and keep terminal output behavior in `monitor` unchanged unless explicitly requested.
- Parser mission extraction now includes destination, issuing/target faction, reward/donation/fine, influence/reputation, wing, expiry, trade commodity/count, and massacre kill-count fields from Journal mission lifecycle events.
- `CargoDepot` is promoted from broad raw event to typed event because trade/mining/source-return progress needs `MissionID`, collected/delivered counts, total delivery target, and cargo names.
- `MissionTracker` classifies accepted missions into `Massacre`, `Trade`, or `Other`, stores origin context from `LoadGame`, `Location`, and `FSDJump`, tracks mission state transitions, updates trade progress from `CargoDepot`, and updates massacre kill progress from `Bounty.VictimFaction`.
- This mirrors the useful ODEliteTracker layering: Journal parser -> mission store/tracker -> later dashboard aggregation. Do not add UI aggregation directly to parser types.
- `Docked` and `Undocked` are promoted to typed events for mission origin context: docking captures station/market, undocking clears station/market while preserving current system.
- `Missions.Active` preserves whether the field was present. An explicit empty active list is authoritative and clears active/redirected tracked missions; a missing `Active` field should not wipe state.
- Massacre bounty progress follows the ODEliteTracker-style stack rule: one matching active mission per issuing faction progresses for a valid bounty. Ignore zero-reward bounty records, suit targets, `faction_none`, and generic pirate faction sentinels.
- Completion reward handling treats `Reward: 0` plus `Donated` as negative donated value; failed/abandoned tracked missions set reward to zero.
