# GUI WebUI Tauri Todo 5 Evidence Coverage Fix

Date: 2026-06-20

## Context

The final adversarial gate rejected Todo 5 for evidence-only blockers, not source/test behavior blockers:

- `task-5-code-review.md` did not explicitly document `omo:programming` and `omo:remove-ai-slops` review coverage.
- `task-5-pure-loc.txt` claimed every touched/new Rust file but only listed the main-refactor subset.

## What changed

Only allowed evidence/knowledge files were updated:

- `.omo/evidence/gui-webui-tauri/task-5-pure-loc.txt`
- `.omo/evidence/gui-webui-tauri/task-5-code-review.md`
- `.omo/evidence/gui-webui-tauri/task-5-manual-qa-matrix.md`
- `.omo/evidence/gui-webui-tauri/task-5-evidence-coverage-fix.txt`
- `.omo/knowledges/gui-webui-tauri-task-5-evidence-coverage-fix-2026-06-20.md`

No product code, tests, UI, docs, plan, or ledger files were edited.

## Useful current facts

Final verification pure LOC table:

- `src/main.rs`: 5
- `src/app.rs`: 153
- `src/app/cli.rs`: 121
- `src/app/cli/tests.rs`: 167
- `src/app/runtime.rs`: 243
- `src/app/runtime/terminal.rs`: 249
- `src/app/feed.rs`: 97
- `src/app/missions.rs`: 183
- `tests/runtime_service.rs`: 128
- `src/app/runtime/types.rs`: 69
- `src/app/runtime/delivery.rs`: 182
- `src/app/runtime/delivery_debug.rs`: 100
- `src/app/runtime/paths.rs`: 54
- `src/app/status.rs`: 178
- `src/app/session.rs`: 110
- `src/app/snapshot.rs`: 42
- `src/app/display.rs`: 50
- `src/app/config.rs`: 203

Maintenance warning:

- `src/app/runtime/terminal.rs` is at 249 pure LOC.
- `src/app/runtime.rs` is at 243 pure LOC.
- Future changes should split terminal prompt/rendering, replay delivery, watch delivery, snapshot projection, or preload processing before adding logic.

## Evidence wording requirements satisfied

`task-5-code-review.md` now explicitly covers:

- `omo:programming`: typed boundaries, DTO privacy, `line_safe` sanitation, no product-path unwrap/panic expansion, runtime extraction, terminal-only replay.
- `omo:remove-ai-slops`: no tautological/deletion-only tests, no implementation-mirroring assertions, no false confidence, no scope drift, and module-size/maintenance burden.

`task-5-manual-qa-matrix.md` now uses a `manualQa` matrix with:

- `surfaceEvidence`
- `adversarialCases`
- `artifactRefs`

The evidence-only command transcript is saved in `.omo/evidence/gui-webui-tauri/task-5-evidence-coverage-fix.txt`.
