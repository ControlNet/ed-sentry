# Matrix user identity discovery

As of 2026-06-29, Matrix configuration no longer supports or documents a `user_id` key.

Runtime Matrix delivery requires only `homeserver`, `room_id`, and `access_token` for identity and target selection. The account identity is discovered at startup by calling `GET /_matrix/client/v3/account/whoami` with the configured bearer access token before restoring the Matrix SDK session.

The whoami response provides `user_id` and may provide `device_id`. When `device_id` is present, use it for the restored session. If an older homeserver omits `device_id`, fall back to the existing fixed device id (`EDAFKDASHBOARD`) so legacy behavior remains usable without reintroducing a user-facing config key.

Do not add `user_id` back to TOML examples, WebUI config API payloads, frontend form state, or config write paths.
