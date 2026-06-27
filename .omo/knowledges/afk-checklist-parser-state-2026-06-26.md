# AFK checklist parser state notes

Date: 2026-06-26

Context: Todo 2 of `.omo/plans/afk-checklist-watcher.md` added the parser/state/view-model foundation for `Status.json` and `Cargo.json` without wiring snapshots or runtime watchers.

Findings:

- `src/app/afk_checklist.rs` owns the companion JSON boundary and view row derivation. It parses only `Status.json` `Flags`/`Pips` and `Cargo.json` `Vessel`/`Count`/`Inventory`.
- The public view rows are stable for later snapshot wiring: `hardpoints_deployed`, `engine_pips_zero`, and `cargo_loaded` with `pass`/`fail`/`unknown` state and `Status.json`/`Cargo.json`/`unknown` source serialization.
- The parser converts malformed, missing, unreadable-equivalent inputs to `unknown` through `AfkChecklistState::unknown()` or `from_optional_companion_json`; it does not panic or reuse the Journal line parser.
- Required evidence for the parser surface is saved in `.omo/evidence/afk-checklist-watcher/task-2-afk-checklist-tests.txt` and `.omo/evidence/afk-checklist-watcher/task-2-malformed-json.txt`.
