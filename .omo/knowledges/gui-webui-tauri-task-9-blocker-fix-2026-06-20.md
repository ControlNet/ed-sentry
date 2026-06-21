# GUI WebUI/Tauri Task 9 Blocker Fix - 2026-06-20

- Todo 9 gate blockers were missing review/manual QA evidence and an oversized `ui/src/components/dashboard/dashboard-shell.tsx`.
- Baseline before edits: `dashboard-shell.tsx` measured 485 pure non-comment LOC while `pnpm --dir ui typecheck` passed.
- The shell was split by responsibility into:
  - `dashboard-shell.tsx` for page composition.
  - `shell-navigation.tsx` for nav rail controls.
  - `metric-grid.tsx` for session metric derivation/rendering.
  - `event-feed.tsx` for feed rows and severity badges.
  - `mission-panel.tsx` for mission rows and progress bars.
  - `source-panel.tsx` for Journal/Matrix/Web status.
  - `dashboard-status.tsx` for reusable status primitives and connection icons.
  - `dashboard-helpers.ts` for pure display helpers.
- Post-refactor LOC evidence: every touched TS/TSX module is below 250 pure LOC. Artifact: `.omo/evidence/gui-webui-tauri/task-9-module-loc.txt`.
- Adapter boundary behavior is covered by `ui/e2e/adapter-boundary.spec.ts` using Playwright with a typed fake WebSocket and injected Tauri transport.
  - Malformed WebSocket JSON emits degraded connection state.
  - WebSocket `hello` expands to snapshot plus buffered event.
  - Tauri valid load parses and malformed stream payload emits degraded connection state.
- Required verification artifacts were refreshed under `.omo/evidence/gui-webui-tauri/task-9-*`, including typecheck, lint, build, focused mock-dashboard, repeated mock-dashboard, adapter boundary, negative grep, screenshot inspection, git status, and cleanup process logs.
- Non-UI dirty files in this worktree are parallel Todo work; Task 9 blocker fix writes were constrained to `ui/**`, `.omo/evidence/gui-webui-tauri/task-9-*`, and `.omo/knowledges/gui-webui-tauri-task-9-*`.
