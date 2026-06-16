# Task 9 Matrix SDK Delivery

- `src/matrix.rs` owns the production Matrix delivery skeleton and is exported from `src/lib.rs` as `pub mod matrix`.
- Matrix startup with `matrix-sdk` 0.18.0 is: build `Client` from `homeserver_url`, restore `MatrixSession` using `SessionMeta` plus `SessionTokens`, call `sync_once(SyncSettings::default())`, then resolve the configured `OwnedRoomId` with `Client::get_room`.
- The fixed SDK device ID comes from `MatrixRuntimeConfig::device_id()` and currently remains `EDAFKDASHBOARD`.
- Matrix notification bodies are assembled only from optional mention prefix, optional emoji, and `Notification.remote_text`, then sanitized through `line_safe`; raw Journal payloads are not referenced by the Matrix module.
- Level `>= 2` notifications only carry Matrix `m.mentions` metadata when `mention_user_id` is configured; otherwise they are normal plain text messages.
- Status publishing sends one original status event and stores that original event ID; later status publishes create edits against the original ID using `Room::make_edit_event` and do not replace the stored original ID with edit event IDs.
- Unit tests use an internal fake `MatrixRoomSender`, so formatting, mentions metadata, edit semantics, and token redaction are covered without network or credentials.
