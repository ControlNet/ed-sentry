# GUI WebUI Tauri Task 12 WebUI QA

- Todo 12 WebUI QA coverage lives in `ui/e2e/webui-qa-gates.spec.ts`.
- Playwright runs against production Vite preview by default through `ui/playwright.config.ts`, whose web server command builds and previews with `VITE_DASHBOARD_ADAPTER=mock`.
- Required Todo 12 responsive artifacts:
  - `task-12-mobile.png` from `@responsive-mobile` at 375px.
  - `task-12-tablet.png` from `@responsive-tablet` at 768px.
  - `task-12-desktop.png` from `@responsive-desktop` at 1280px.
- `@keyboard-focus` proves Tab/Shift+Tab reaches Dashboard, Config, Journal folder, Save, and Cancel, and records focus ring box-shadow observables.
- `@reduced-motion` uses Playwright `emulateMedia({ reducedMotion: "reduce" })` and verifies `transitionDuration=0s` plus spinner `animationDuration=0.001s`.
- Mock-only state fixtures are selected by `?mock_state=empty|loading|error` in `ui/src/adapters/mock.ts`; they are for browser QA and do not expose production user data.
- `pnpm --dir ui audit:react` runs `react-doctor --json --no-score -y`; the current gate exits 0 with warnings but no errors.
- Real-browser Lighthouse was not wired because `playwright-lighthouse` and `lighthouse` are not installed; the Todo 12 evidence records this as an explicit limitation and does not use Lighthouse CLI.
- Existing Axum production served responsive smoke path is `scripts/smoke-webui.sh --scenario responsive --evidence .omo/evidence/gui-webui-tauri/task-12-axum-responsive.txt`.
