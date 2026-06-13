# Broad Journal event support

- Unsupported Elite Dangerous Journal events are now parsed into specific raw-preserving `JournalEvent::<EventName>` variants instead of collapsing recognized-but-unmodeled events into broad categories or `Unknown`.
- `RawJournalEvent` stores `timestamp`, `event`, and the full `serde_json::Value` payload. `TypedJournalEvent` is currently an alias for that raw-preserving shape, so every enumerated event has a concrete enum variant without pretending we have full per-field Journal schemas yet.
- `JournalEvent::raw_payload()` exposes preserved JSON for specific typed broad events and truly unknown future events.
- Existing Phase 1 field-rich variants and monitor/state behavior remain unchanged; newly typed broad events are ignored by `SessionState` unless promoted to richer behavior later.
- Parser tests iterate every broad event-name table and assert each event parses as non-`Unknown` with nested raw payload preservation. The public parser API test locks `DockingGranted` as a concrete `JournalEvent::DockingGranted` variant.
- `stream_journal_file()` provides a no-collection ingestion path for replay/raw-ingest work. Replay now uses streaming passes instead of holding all replay records in memory while preserving startup commander output order.
- The known raw event names are maintained through one `known_raw_journal_events!` macro table that expands the concrete `JournalEvent` variants, parser dispatch, helper matching, and parser coverage tests. Avoid reintroducing separate hand-maintained lists for these names.
- `stream_journal_file()` callbacks are fallible and short-circuit immediately. `preload_journal_file_with_options()` and streaming share the same private record-reading loop so offsets, UTF-8 handling, line numbers, and parser-error wrapping do not drift.
