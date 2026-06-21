# GUI WebUI Tauri Task 13 Gate Fix

Date: 2026-06-20

## Fixed Gate Blockers

- `src/app/runtime/desktop.rs` no longer uses the no-op `let _ = Arc::strong_count(&self.delivery);` in `Drop`.
- `DesktopRuntime` no longer stores a `delivery` field only for lifetime signaling. The spawned monitor task owns the delivery `Arc`, and `Drop` aborts `monitor_task`.
- `ui/src/adapters/tauri.ts` preserves safe string rejections from Tauri command promises as `DashboardAdapterError` messages.
- Tauri command rejection strings are redacted before surfacing when they contain common sensitive shapes:
  - bearer token-shaped strings
  - `access_token`, `token`, `secret`, `password`, or `authorization` assignments
  - common Unix/Windows private user paths
- `ui/e2e/adapter-boundary.spec.ts` covers string rejections for `loadSnapshot`, `loadConfig`, and `saveConfig`.

## Evidence Facts

- Focused failing-first proof initially failed because the adapter returned generic `non-Error value` messages for string rejections.
- Final focused command passed after the fixture wording change:
  - `pnpm --dir ui test:e2e -- --project=chromium --grep "tauri adapter .*string errors"`
  - Artifact: `.omo/evidence/gui-webui-tauri/task-13-gate-fix-tauri-string-rejections.txt`
- `pnpm --dir ui lint` was rerun after the final fixture wording change.
- Full Rust/Tauri/UI gates passed before the final fixture wording-only test change:
  - `cargo test --all`
  - root and Tauri clippy with `-D warnings`
  - direct Tauri crate tests
  - `pnpm --dir ui typecheck`
  - `pnpm --dir ui build`
  - `pnpm --dir ui tauri build`
- The Tauri release binary was recorded at `ui/src-tauri/target/release/ed-sentry-gui`.

## Review Notes

- The string rejection tests assert public adapter behavior and do not call private helper functions, so they are not implementation-mirroring tests.
- The no-op Drop fix should remain structural: if future code needs desktop delivery lifetime ownership, model that with an actual owner or task handle, not a count read.
- Reports refreshed:
  - `.omo/evidence/gui-webui-tauri/task-13-code-review.md`
  - `.omo/evidence/gui-webui-tauri/task-13-manual-qa-matrix.md`
