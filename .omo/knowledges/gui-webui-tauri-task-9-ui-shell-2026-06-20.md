# GUI WebUI/Tauri Task 9 UI shell - 2026-06-20

- Todo 9 frontend work is scoped to `ui/**` plus evidence. Rust runtime files were not edited.
- The shared frontend adapter boundary lives under `ui/src/adapters/`.
  - `types.ts` mirrors Todo 3 DTO shapes with Zod boundary parsing for `AppSnapshot`, session, missions, feed, Journal, Matrix, and Web status views.
  - `mock.ts` and `mock-data.ts` provide sanitized fixture-like data so the dashboard renders without backend or raw Journal/private token content.
  - `web.ts` provides HTTP snapshot loading through Ky v2 (`baseUrl`, timeout, no retry) and WebSocket message-envelope parsing.
  - `tauri.ts` is an adapter-only placeholder using injected transport functions. It imports no `@tauri-apps/api` and shared components import no Tauri APIs.
- Dashboard state is in `ui/src/store/dashboard-store.ts` and owns adapter lifecycle, connection state, snapshot refresh, and live feed item application.
- `ui/src/components/dashboard/dashboard-shell.tsx` renders the Todo 9 shell: responsive nav, connection state, metrics, event feed, mission progress, and Journal/Matrix/Web status. Icons are Lucide only.
- `ui/scripts/playwright.mjs` strips the literal leading `--` that pnpm forwards in this environment, so the required command `pnpm --dir ui test:e2e -- --project=chromium --grep "@mock-dashboard"` applies Playwright args correctly.
- Evidence:
  - `.omo/evidence/gui-webui-tauri/task-9-dashboard-shell.txt`
  - `.omo/evidence/gui-webui-tauri/task-9-dashboard-shell.png`
  - `.omo/evidence/gui-webui-tauri/task-9-typecheck.txt`
  - Extra responsive screenshots: `task-9-dashboard-mobile.png`, `task-9-dashboard-tablet.png`.
