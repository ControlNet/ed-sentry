# Tauri titlebar drag regions

Date: 2026-06-22

Tauri v2 `data-tauri-drag-region` applies drag behavior only to the element that directly carries the attribute. Child elements that should also drag need their own `data-tauri-drag-region`; interactive controls should remain outside drag behavior with explicit no-drag styling/markers.

For the ed-sentry frameless dashboard top bar:

- The titlebar header, primary nav container, and the nav list/gap container should be draggable.
- Workspace tab buttons and window controls should be no-drag so clicks remain interactive.
- `app-region: drag` on `[data-tauri-drag-region]` and `app-region: no-drag` on `[data-titlebar-no-drag]` are required for the Windows WebView path documented by Tauri.
- Regression coverage lives in `ui/e2e/tauri-window-chrome.spec.ts`; the hitmap test should assert both debug markers and actual `data-tauri-drag-region` attributes for the nav/list regions.

Observed root cause in the previous implementation: the center nav had only `data-titlebar-drag-region="primary-nav"` for debug visualization, not `data-tauri-drag-region`, so the top bar looked highlighted but had non-draggable gaps in native Tauri hit testing.
