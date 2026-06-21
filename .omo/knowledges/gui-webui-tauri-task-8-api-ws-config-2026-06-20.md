# GUI WebUI/Tauri Task 8 API, WebSocket, and Config Write

Date: 2026-06-20

- WebUI backend routes now live in `src/web/policy.rs`.
- Implemented HTTP endpoints:
  - `GET /api/health`
  - `GET /api/snapshot`
  - `GET /api/config`
  - `PUT /api/config`
  - `GET /api/web/status`
  - `GET /api/matrix/status`
  - `GET /api/web/policy`
- Implemented WebSocket endpoints at both `/api/events` and `/ws`.
- WebSocket protocol messages are JSON envelopes with explicit `type` and `version` fields. Current types are `hello`, `snapshot`, `event`, and `error`.
- The WebUI server now receives `MonitorRuntime::event_store()`, so HTTP snapshots and WebSocket bootstrap/live updates use the same sanitized event buffer populated by the monitor runtime.
- `GET /api/snapshot` returns the existing `AppSnapshot` shape plus a top-level `events` alias for Todo 8 compatibility. The existing frontend adapter can continue using `event_feed`.
- Config view uses `EditableConfigView`; Matrix tokens are represented only by `access_token_present`.
- Config writes are handled by `AppConfig::write_update_to_source` in `src/config/write.rs`, using `ConfigSource` and `ConfigPath` from Todo 2.
- `toml_edit` is used for config writes so existing comments and unknown keys are preserved when an existing config file is edited.
- Matrix token write rules:
  - If `access_token_replacement` is omitted or null, the existing token is preserved.
  - If `access_token_replacement` is provided, the token key is replaced.
  - If `clear_access_token` is true, the token key is removed.
  - No API response returns the raw token value.
- First milestone write safety:
  - `PUT /api/config` is enabled only for loopback-bound WebUI servers.
  - Non-loopback server binds return `403` for config mutation.
  - Host and Origin are validated for state-changing routes.
  - CORS allows only loopback origins.
- `scripts/probe-websocket.mjs` is a minimal Node built-in WebSocket probe for smoke testing; no `websocat` dependency is required.
