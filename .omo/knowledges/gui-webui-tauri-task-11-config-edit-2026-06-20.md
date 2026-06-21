# GUI WebUI Tauri Task 11 Config Editing

Date: 2026-06-20

## Decisions

- Config editing is exposed inside the existing dashboard shell via the `Config` navigation item, not as a landing page or separate router.
- The frontend config boundary lives in `ui/src/adapters/config.ts` with Zod response parsing and a typed `EditableConfigUpdate` payload.
- `DashboardAdapter` now includes `loadConfig` and `saveConfig`. The Web adapter maps those to `GET/PUT /api/config`; the mock adapter uses sanitized in-memory config for UI/e2e; the Tauri adapter explicitly returns "not available yet" until Todo 13.
- Matrix token handling is write-only:
  - API view exposes only `access_token_present`.
  - Leaving replacement blank sends `access_token_replacement: null` and preserves the backend token.
  - Non-empty replacement sends the replacement value and sets token-present true after save.
  - Explicit clear sends `clear_access_token: true` and removes the token.
  - The UI renders `Token stored` / `No token stored`, never the current raw token.
- Backend config writes still use the Todo 8 `toml_edit` path for comment/unknown-key preservation, now written through a sibling temp file plus rename.
- Journal editing exposes `folder` and `recent_files`. Single-file Journal source is not in the editable backend DTO.

## Evidence

- Failing-first: `.omo/evidence/gui-webui-tauri/task-11-failing-first.txt`
- Browser save/reload: `.omo/evidence/gui-webui-tauri/task-11-config-edit.txt`, `.omo/evidence/gui-webui-tauri/task-11-config-edit.png`
- Token masking: `.omo/evidence/gui-webui-tauri/task-11-token-mask.txt`, `.omo/evidence/gui-webui-tauri/task-11-token-mask.png`
- Full verification: `.omo/evidence/gui-webui-tauri/task-11-verification.txt`
- Secret scan: `.omo/evidence/gui-webui-tauri/task-11-secret-scan.txt`

## Follow-Up

- Todo 12 should run broader responsive visual/performance QA.
- Todo 13 should replace the Tauri adapter config stubs with native command bindings.
- Existing oversized Rust files (`src/config/write.rs`, `tests/webui.rs`) should be split in a dedicated refactor, not as part of config UI scope.
