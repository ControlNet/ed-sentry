# GUI WebUI Tauri Task 7 Evidence Coverage Fix - 2026-06-20

The Todo 7 final gate blocker was evidence-only. Source/functionality/smoke/Cargo gates had passed, but the existing `task-7-code-review.md`, `task-7-manual-qa-matrix.md`, and `task-7-source-review.txt` did not explicitly label `omo:programming` review coverage or `omo:remove-ai-slops` overfit/slop criterion coverage.

Files refreshed:
- `.omo/evidence/gui-webui-tauri/task-7-code-review.md`
- `.omo/evidence/gui-webui-tauri/task-7-manual-qa-matrix.md`
- `.omo/evidence/gui-webui-tauri/task-7-source-review.txt`
- `.omo/evidence/gui-webui-tauri/task-7-evidence-coverage-fix.txt`

No product code, tests, UI, Cargo files, scripts, plan, or ledger files were edited.

Important evidence references:
- Gate rerun HTTP artifacts: `task-7-gate-web-root.http`, `task-7-gate-packaged-assets.http`.
- Gate rerun occupied-port artifact: `task-7-gate-web-port-warning.txt`.
- Original full verification log: `task-7-verification.txt`.
- Cleanup/process check: `task-7-cleanup-process-check.txt`.
- Pure LOC evidence: `task-7-pure-loc.txt`.

Explicit coverage added:
- `omo:programming`: typed WebUI status/error boundaries, Axum/static serving boundary, config/env/path validation, no production unwrap/panic expansion, watch/replay startup separation, bind failure warning semantics, non-localhost disabled write policy, dependency/features sanity.
- `omo:remove-ai-slops`: no overfit smoke tests, no tautological/deletion-only tests, no implementation-mirroring-only assertions, no false confidence from HTTP logs alone, no scope drift into UI/Todo 8 protocol, no needless abstraction/excessive complexity, and module/script LOC/maintenance burden.

Current source review conclusions:
- Static assets resolve from `ED_SENTRY_WEBUI_DIST`, executable sibling `webui/`, then repo `ui/dist`, with `index.html` required.
- Source grep for `ui/src` in reviewed Task 7 files returned no matches.
- `/ws` remains a 501 placeholder; no WebSocket protocol was implemented.
- `WebEndpointPolicy::placeholder_disabled` keeps `state_changing_enabled` false.
- No WebUI auth, Authorization, cookie, token, login, or public remote write mode was found in scoped WebUI/runtime files.
- `run_watch` starts WebUI through `start_webui`; `run_replay` does not.
- Bind failures become warning status and terminal warnings; gate rerun artifact shows warning plus continued monitor output.

Adversarial classes closed in evidence:
- dirty_worktree
- stale_state
- misleading_success_output
- generated_cached_artifacts
- overfit_smoke_tests
- tautological_or_deletion_only_tests
- implementation_mirroring_only_assertions
- false_confidence_from_http_logs
- scope_drift_into_ui_or_todo8_protocol
- product_unwrap_panic_expansion
