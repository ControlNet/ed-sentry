Session ship display must prefer Elite Dangerous `Ship_Localised` over the raw `Ship` key.

Root cause fixed on 2026-06-23: `SessionState` applied `LoadGame.Ship` then `LoadGame.Ship_Localised`, but later raw-only `Loadout.Ship` events overwrote the human display name. In local journals this made the WebUI `SHIP` row show `type9_military` after `LoadGame` had already provided `Ship_Localised = "Type-10 Defender"`.

Required behavior:

- Track the raw ship key separately from the display string.
- When a later `Loadout` only repeats the same raw ship key, preserve the existing localised display string.
- If the raw ship key changes and no localised value is present, update the display to the new raw key so ship changes remain visible.

Verification signal used for this fix:

- A live `/api/snapshot` probe using `LoadGame { Ship: "Type9_Military", Ship_Localised: "Type-10 Defender" }` followed by raw-only `Loadout { Ship: "type9_military" }` returned `session.ship=Type-10 Defender`.
