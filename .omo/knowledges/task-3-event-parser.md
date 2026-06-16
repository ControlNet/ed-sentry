# Task 3 Journal Event Parser

- Public parser API lives in `ed_sentry::event`: `parse_journal_line(&str)` for line-delimited Journal input and `parse_journal_value(&serde_json::Value)` for already-decoded JSON.
- `JournalEvent` preserves `DateTime<Utc>` and the raw Journal event name for every known Phase 1 variant and for `JournalEvent::Unknown { timestamp, event }`.
- Unknown valid events are non-fatal; malformed JSON returns `JournalParseError::MalformedJson`, missing `event` returns `JournalParseError::MissingEvent`, and missing/invalid timestamps are separate recoverable errors.
- Optional field structs currently cover text (`From`, `From_Localised`, `Message`, `Channel`), targeting (`TargetLocked`, `ScanStage`, `PilotName`, `LegalStatus`), bounty/kill bond rewards and factions, mission ID/name/localised name, shields/hull damage, cargo ejection, fuel reservoir/main, and music track.
- Fixture coverage is synthetic and sanitized. `Rank`, `Progress`, `Loadout`, `SupercruiseEntry`, and `FSDJump` are covered with inline synthetic test JSON rather than fixture files.
