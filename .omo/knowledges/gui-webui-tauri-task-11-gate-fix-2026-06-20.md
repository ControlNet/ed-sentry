# GUI WebUI Tauri Task 11 Gate Fix

Date: 2026-06-20 UTC

## Config API Error Mapping

Frontend-safe `/api/config` write error mapping lives in `src/web/policy/config_api.rs`.

- `ConfigWriteError::MalformedToml` maps to code `malformed_config` and a fixed safe message.
- `ConfigWriteError::Io` maps to code `config_write_failed` and a fixed safe message.
- `ConfigWriteError::UnsafeRemoteBind`, `NoWritableTarget`, and `Blocked` also map to fixed strings and do not echo host/source/path details.
- Internal `ConfigWriteError` display strings in `src/config/write.rs` still include paths for internal diagnostics; the HTTP layer must not serialize them.

## Deterministic Write-Failure Test

Use a first-save target beneath an existing regular file to force a real filesystem write failure through the live route without relying on chmod behavior:

- create `not-a-directory` as a file;
- use target `not-a-directory/config.toml`;
- send live `PUT /api/config`;
- assert HTTP 500 safe body, target file absent, and blocking file unchanged.

The tests are:

- `api::webui_api_config_update_malformed_toml_error_is_frontend_safe`
- `api::webui_api_config_update_write_failure_error_is_frontend_safe_without_partial_write`

## Split Boundaries

Task 11 oversized files were split without `SIZE_OK` exceptions:

- `src/config/write.rs`: orchestration only; TOML mutation in `src/config/write/apply.rs`; atomic write in `src/config/write/atomic.rs`.
- `src/web/policy.rs`: route policy/validation only; config API in `src/web/policy/config_api.rs`; WebSocket session/envelope in `src/web/policy/ws.rs`.
- `src/app/config.rs`: conversion impls only; DTO definitions in `src/app/config/types.rs`.
- `tests/webui.rs`: module hub; scenario modules under `tests/webui/`.
