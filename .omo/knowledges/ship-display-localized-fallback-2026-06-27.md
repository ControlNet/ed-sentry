# Ship display localized fallback

- Elite Journal `LoadGame`/`Loadout` samples may provide both a user ship name in `ShipName` and a ship model display name in `Ship_Localised`. Preserve both fields through parsing; WebUI snapshot session ship display should render `ShipName (Ship_Localised)` when both are present and different.
- If only one display field is present, display that field. If neither `ShipName` nor `Ship_Localised` is present, fall back to the raw `Ship` id without overwriting an existing display for the same raw ship key.
- `ShipyardSwap` carries `ShipType` and `ShipType_Localised`; session state must apply it like `LoadGame`/`Loadout` so WebUI snapshots do not keep an old ship value or raw id after swapping ships.
- Regression coverage lives in `tests/session_state.rs` with the sanitized `journal_damage_fighter.log` fixture and an explicit `ShipyardSwap` state test.
