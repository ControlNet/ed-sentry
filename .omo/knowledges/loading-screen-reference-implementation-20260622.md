# Loading screen reference implementation

Date: 2026-06-22

## Context

The loading page design reference is `reference-design/design_loading.tsx`. Treat it as read-only input. Do not edit it when implementing the app.

## Implementation

The app loading state is now implemented by `ui/src/components/dashboard/loading-screen.tsx` and used from `ui/src/App.tsx` when `snapshot === null`.

The component preserves the reference direction:

- tactical grid background through the existing `.bg-tactical` system class
- central circular SVG progress indicator
- orange tactical glow on the progress ring and status text
- staged ED-Sentry startup labels
- small linear progress indicator
- live adapter detail text from `connection.detail`

The progress is intentionally bounded at 96% while waiting for the real snapshot. It does not fake completion; the loading screen unmounts when the dashboard receives its first snapshot.

The independent background glow orb from the reference was not ported because project UI rules disallow decorative orb/blob backgrounds. The visual emphasis instead comes from the progress ring itself.

## Regression coverage

`ui/e2e/dashboard-smoke.spec.ts` has `@loading-screen renders the tactical startup visual while awaiting a snapshot`.

It uses `/?mock_state=loading` to keep the adapter unresolved and captures:

- `.omo/evidence/gui-webui-tauri/loading-screen.png` at 1280x720
- `.omo/evidence/gui-webui-tauri/loading-screen-tablet.png` at 768x1024
- `.omo/evidence/gui-webui-tauri/loading-screen-mobile.png` at 375x812

## Verification

- `pnpm --dir ui lint`
- `pnpm --dir ui typecheck`
- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui build`
- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui exec playwright test e2e/dashboard-smoke.spec.ts e2e/tauri-window-chrome.spec.ts e2e/reference-redesign.spec.ts`
