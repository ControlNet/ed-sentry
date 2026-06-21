# GUI/WebUI Task 3 App DTOs

- `ed_sentry::app` is the shared DTO boundary for WebUI/Tauri-facing dashboard payloads.
- Snapshot DTOs live under `src/app/snapshot.rs` and are re-exported from `src/app.rs`: `AppSnapshot`, `SessionView`, `MissionListView`, `MissionView`, `NotificationView`, `EventFeedItem`, `JournalSourceView`, `RateView`, and `ValueDisplay`.
- Config-edit DTOs live under `src/app/config.rs`. `EditableConfigView::from_runtime_config` maps current `RuntimeConfig` sections including `[web]`.
- `MatrixConfigView` exposes `access_token_present` only. `access_token_replacement` is deserialize-capable for future edits but `skip_serializing`, so runtime serialization never echoes `MatrixConfig.access_token`.
- Service status DTOs live under `src/app/status.rs`: `MatrixStartupStatus`/`WebStartupStatus` are app-layer startup seams, converted into `MatrixStatusView`/`WebStatusView`.
- Journal DTOs expose source path metadata only; no raw Journal line or `raw_payload` content is serialized through the app module.
- As of this task, `cargo test --lib app` and the two evidence tests pass. `cargo test --all` is blocked by non-Todo-3 `RuntimeConfig` literals in `tests/journal_discovery.rs` and `src/main.rs` tests missing the Todo 2 `web` field.
