# GUI WebUI Tauri Task 10 Live Dashboard Decisions

Date: 2026-06-20

## Summary

Todo 10 implemented the first live WebUI dashboard milestone and wired the production Web adapter to the Axum backend.

## Decisions

- Production UI defaults to the Web adapter, while local Playwright dev tests explicitly build with `VITE_DASHBOARD_ADAPTER=mock`.
- The Web adapter now fetches both `/api/snapshot` and `/api/config` and subscribes to `/api/events`.
- WebSocket `hello` envelopes apply the snapshot and buffered `event_feed`; store-side event merging deduplicates repeated event ids.
- Dashboard regions are split by operational responsibility: session context, connection state, combat metrics, health/fuel, warning rail, recent event feed, mission progress table, Journal source, Matrix status, and Web status.
- The health/fuel panel derives fuel from sanitized event feed text because the current snapshot DTO has no dedicated fuel field.
- Journal source display sanitizes absolute folder paths and raw Journal filenames before rendering.
- Matrix status supports disabled/warning/degraded states and never displays raw access tokens.

## Smoke Evidence

- Live dashboard: `.omo/evidence/gui-webui-tauri/task-10-live-dashboard.txt` and `.omo/evidence/gui-webui-tauri/task-10-live-dashboard.png`.
- Buffered events: `.omo/evidence/gui-webui-tauri/task-10-buffered-events.txt` and `.omo/evidence/gui-webui-tauri/task-10-buffered-events.png`.
- Responsive: `.omo/evidence/gui-webui-tauri/task-10-responsive.txt` plus 375, 768, and 1280 px screenshots.

## Notes For Later Tasks

- Todo 11 should add config editing without relying on the current status panels as form controls.
- A future backend DTO field for fuel would make the health/fuel panel less dependent on event history.
- The Matrix warning state in smoke tests is intentional; Todo 10 must not require a real Matrix homeserver.
