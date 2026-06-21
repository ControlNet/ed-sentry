# GUI/WebUI/Tauri plan

- The executable work plan is `.omo/plans/gui-webui-tauri.md`.
- The durable planning draft is `.omo/drafts/gui-webui-tauri.md`.
- Plan scope: shared React/Vite/shadcn frontend under `ui/`, Axum WebUI backend, reusable Rust application service, editable sanitized config, backend recent-event buffer, WebSocket realtime transport, and separate `ed-sentry-gui` Tauri desktop entry.
- Confirmed startup model: no `--webui` flag. `[web] enabled = true` starts the optional WebUI server in watch-capable CLI and desktop GUI runtimes.
- Confirmed runtime model: CLI and desktop GUI are different entry points over the same Rust application services; they must both honor config-enabled Web/Matrix behavior where supported.
- Confirmed exclusions: no GUI replay, no historical database, no auth/public remote mode, no chart library in the first milestone, and no Matrix command/automation work.
- Verification emphasis: Rust tests/lints, frontend typecheck/build, HTTP/WebSocket surface checks, Playwright browser screenshots, desktop/Tauri smoke/build, replay isolation, and secret/token leak scans.
