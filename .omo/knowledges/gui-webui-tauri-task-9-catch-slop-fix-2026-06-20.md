# GUI WebUI/Tauri Task 9 Catch Slop Fix - 2026-06-20

## Context

Task 9 had a remaining slop blocker after the strict catch-narrowing fix. The no-excuse checker passed, but three catch sites were checker-shaped:

- `ui/src/adapters/web.ts` load-snapshot catch had `Error` and non-`Error` branches that both threw `formatAdapterError("web", error)`.
- `ui/src/adapters/tauri.ts` load-snapshot catch had `Error` and non-`Error` branches that both threw `formatAdapterError("tauri", error)`.
- `ui/src/adapters/tauri.ts` stream-payload catch had a ternary where both arms called `formatAdapterError("tauri", error)`.

## Change

Only `ui/src/adapters/web.ts` and `ui/src/adapters/tauri.ts` production code changed.

The real `Error` path still uses `formatAdapterError(...)`, preserving the previous message behavior for Zod/ky/parser errors. The non-`Error` path now creates a distinct `DashboardAdapterError` with an adapter-specific message:

- Web load non-`Error`: `Web adapter failed with a non-Error value`
- Tauri load non-`Error`: `Desktop adapter failed with a non-Error value`
- Tauri stream non-`Error`: `Desktop payload parser failed with a non-Error value`

No new helper was added because the existing code needed only three local branch fixes and adding a helper would have been unnecessary abstraction for this pass.

## Verification Artifacts

- Failing-first source capture: `.omo/evidence/gui-webui-tauri/task-9-catch-slop-failing-first.txt`
- Source slop review: `.omo/evidence/gui-webui-tauri/task-9-catch-slop-source-review.txt`
- No-excuse checker: `.omo/evidence/gui-webui-tauri/task-9-no-excuse-check.txt`
- Typecheck: `.omo/evidence/gui-webui-tauri/task-9-typecheck.txt`
- Lint: `.omo/evidence/gui-webui-tauri/task-9-lint.txt`
- Build: `.omo/evidence/gui-webui-tauri/task-9-build.txt`
- Adapter E2E: `.omo/evidence/gui-webui-tauri/task-9-adapter-boundary.txt`
- Mock dashboard E2E: `.omo/evidence/gui-webui-tauri/task-9-dashboard-shell.txt`
- Negative greps: `.omo/evidence/gui-webui-tauri/task-9-negative-greps.txt`
- LOC check: `.omo/evidence/gui-webui-tauri/task-9-module-loc.txt`
- Git status: `.omo/evidence/gui-webui-tauri/task-9-git-status.txt`
- Cleanup process check: `.omo/evidence/gui-webui-tauri/task-9-cleanup-processes.txt`
- Refreshed code review: `.omo/evidence/gui-webui-tauri/task-9-code-review.md`

## Notes

The passing adapter-boundary E2E run confirms malformed WebSocket JSON still produces a degraded connection and malformed Tauri stream payloads still produce a degraded connection. The source review grep confirms the old identical catch-branch and identical ternary patterns are absent.
