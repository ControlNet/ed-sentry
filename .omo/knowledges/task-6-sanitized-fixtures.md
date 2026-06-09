# Task 6 Sanitized Fixtures

- Journal fixtures live under `tests/fixtures/` and are synthetic, minimized, line-delimited JSON.
- Raw local Journals are reference-only input and must not be committed.
- `journal_malformed_unknown.log` intentionally has exactly one malformed JSON line plus one valid unknown event named `FixtureUnknownEvent`.
- `tests/fixture_sanity.rs` is the fixture safety gate: it validates JSON lines, required `timestamp` and `event` fields, the malformed-line count, and absence of raw local path/token patterns in fixture logs.
- Required Task 6 evidence files are `.omo/evidence/task-6-fixtures.txt`, `.omo/evidence/task-6-raw-paths.txt`, and `.omo/evidence/task-6-secret-scan.txt`.
