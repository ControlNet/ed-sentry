# Loading screen titlebar drag region

Date: 2026-06-23

The Tauri dashboard loading screen now receives `isTauri` from `App` before the first snapshot exists. When the desktop adapter is active, the top 56px of `LoadingScreen` carries `data-tauri-drag-region` and `data-titlebar-drag-region="loading-titlebar"`, plus the same `getCurrentWindow().startDragging()` fallback used by the loaded dashboard shell. The existing `?debug_titlebar_drag=1` flag also enables the loading drag strip in browser QA so the attribute and hit area can be inspected before a native Tauri launch.

Keep the drag strip non-interactive and visually transparent so it does not block startup status content below it. Regression coverage lives in `ui/e2e/tauri-window-chrome.spec.ts`, which checks the loading screen source for the Tauri drag attribute and manual drag fallback.
