# Elite Dangerous Journal/status ship-state capability notes

Date: 2026-06-26

Context: investigated whether local Journal and companion files expose power distribution, cargo, ship-launched fighter, and hardpoint deployment state.

Findings:

- The current app discovers/tails only `Journal.*.log` files via `src/journal.rs`; it does not currently ingest companion files such as `Status.json`, `Cargo.json`, or `ModulesInfo.json`.
- `Status.json` is the source for frequently-changing cockpit state. It can include:
  - `Pips`: `[sys, eng, wep]` in half-pips, for current power distribution.
  - `Flags`: bitfield; bit 6 (`0x40`) means hardpoints deployed, bit 24 means in main ship, bit 25 means in fighter, bit 26 means in SRV.
  - `FireGroup`, `GuiFocus`, `Fuel`, and sometimes `Cargo` mass.
- A local `Status.json` sample only exposed `Flags` at the time of investigation, so that snapshot did not show pips or hardpoints deployed, but the file format supports them when the game writes those fields.
- Cargo state is available from `Cargo.json` and from `Cargo` journal events. A local sample showed empty ship cargo after an earlier non-empty cargo event sequence.
- Current cargo value is not directly exposed by `Cargo.json` or `Cargo` startup/update events; they provide cargo names/counts/stolen/mission IDs. `MarketBuy`/`MarketSell` events provide historical transaction prices and `Market.json` provides prices only for the currently-opened market/station, so a reliable live "cargo value > N" checklist should fall back to cargo non-empty unless an explicit heuristic with caveats is desired.
- Ship-launched fighter state is event-derived in journal logs: `LaunchFighter`, `DockFighter`, `FighterDestroyed`, `FighterRebuilt`, and `HullDamage` with `Fighter:true`. Local logs contain all of these. `Status.Flags` bit 25 only indicates the player is currently in a fighter, not necessarily that an NPC SLF is deployed.
- `ModulesInfo.json`/`Loadout` show installed modules, module power draw, priorities, ammo and health, including `PowerDistributor` and hardpoint modules; they do not provide current pips or hardpoint deployed state.
- GUI readiness checklist feasibility: replacing the tactical dashboard `Ship Integrity` panel with `Checklist` is straightforward in the frontend, but the backend must add companion-file ingestion. Checklist mappings should be: hardpoints deployed = `Status.Flags & 0x40 != 0`; cargo ready = `Cargo.json` ship `Count > 0` or non-empty inventory; engine pips zero = `Status.Pips[1] == 0` (Pips are `[sys, eng, wep]` in half-pips). Missing `Status.json`/`Cargo.json` fields should render as unknown/unavailable rather than pass/fail.

Implementation implication: to expose these states in the app, add a companion-file watcher/reader for `Status.json` and `Cargo.json` in addition to the existing journal tail. Keep journal event tracking for fighter lifecycle because companion status does not fully represent NPC fighter deployment/docking/destruction.

Reference project note: ODEliteTracker delegates journal/status tracking to `JournalEventParser`/`EliteJournalReader`. Its app-level `Services/JournalManager.cs` registers event callbacks (`OnJournalEventReceived`, `LiveStatusChange`) and calls `StartWatchingAsync`. The underlying `MagicMau/EliteJournalReader` uses `FileSystemWatcher` for journal logs plus line-level tailing and a 5-second fallback check, and uses a separate `StatusWatcher : FileSystemWatcher` for `Status*.json`. Cargo/Market/NavRoute companion files are exposed as read-on-demand helpers such as `ReadCargoJson`; ODEliteTracker reads cargo on Cargo events rather than owning an independent Cargo.json watcher.
