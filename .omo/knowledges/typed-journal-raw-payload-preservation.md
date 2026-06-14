Typed Journal raw payload preservation
======================================

Context
-------
- The dashboard promotes selected Elite Dangerous Journal events from generic raw storage into typed structs for downstream state/model access.
- Promotion must remain lossless: modeled fields are convenient projections, not replacements for the original Journal JSON.

Decision
--------
- Every dedicated typed event struct stores `raw: Option<serde_json::Value>`.
- Parser-created typed events set `raw: Some(value.clone())` with the complete original Journal object.
- Hand-built test events may use `raw: None` when raw payload is irrelevant to that test.
- `JournalEvent::raw_payload()` returns the stored raw payload for dedicated typed variants, known raw-preserving variants, and `Unknown`.

Regression signal
-----------------
- `tests/event_parser.rs::event_parser_public_api_preserves_raw_payload_for_typed_events` parses a typed `Cargo` event containing an unmodelled nested field and asserts the field is still accessible through `raw_payload()`.

Implication
-----------
- Future Journal event promotion should add typed fields for commonly used data while preserving full raw payload with the same pattern.
