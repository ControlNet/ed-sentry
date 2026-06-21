# GUI WebUI Tauri Task 7 Knowledge

Date: 2026-06-20

Todo 7 implemented the Axum WebUI server and smoke harness.

Key implementation notes:

- `src/web.rs` is a small module facade.
- `src/web/assets.rs` resolves assets in this order: `ED_SENTRY_WEBUI_DIST`, sibling `webui/` beside the executable, then repo-local `ui/dist`.
- `src/web/policy.rs` builds the Axum router. Static files are served with `tower_http::services::ServeDir`. The only WebUI API route added is `/api/web/policy`, which reports that state-changing endpoints are disabled. `/ws` is a deliberate `501 Not Implemented` placeholder.
- `src/web/server.rs` starts the listener and returns `WebStartupStatus::warning` rather than failing the monitor when assets or bind fail.
- Watch mode starts WebUI through `src/app/runtime/web.rs`. Replay does not call WebUI startup.
- Non-localhost binds produce a startup security warning and still expose a disabled state-changing endpoint policy.
- `scripts/smoke-webui.sh` runs root, packaged-assets, and occupied-port scenarios without `websocat`, writes HTTP/warning evidence, and kills child processes.

Final verification artifact: `.omo/evidence/gui-webui-tauri/task-7-verification.txt`.
