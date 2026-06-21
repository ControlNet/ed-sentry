# GUI WebUI Tauri Task 13 Desktop Entry

Date: 2026-06-20

## Implementation Notes

- Added a Tauri v2 app under `ui/src-tauri/` with product/binary name `ed-sentry-gui`.
- Added local UI scripts:
  - `pnpm --dir ui tauri`
  - `pnpm --dir ui tauri:dev`
  - `pnpm --dir ui tauri:build`
- Pinned mature Tauri JS package versions to satisfy the repo's package-age policy:
  - `@tauri-apps/api` `2.11.0`
  - `@tauri-apps/cli` `2.11.2`
- The Tauri Rust crate depends on the root `ed-sentry` library crate by path.
- The shared desktop runtime seam is `ed_sentry::app::runtime::DesktopRuntime`.
- `DesktopRuntime::start` reuses:
  - `MonitorRuntime`
  - `start_webui_silent`
  - `build_watch_delivery_with_terminal`
  - Matrix startup header/delivery/status publication helpers
- Tauri commands:
  - `load_snapshot`
  - `load_config`
  - `save_config`
- Tauri events use `ed-sentry://dashboard` and carry existing `AppLiveUpdate` payloads.
- Frontend Tauri direct API usage is isolated to `ui/src/adapters/tauri.ts`.
- Tauri config editing reuses the existing redacted config DTO/write path; raw Matrix tokens are not returned to the GUI.

## Verification

- `cargo test --lib app::tests::desktop_bootstrap_honors_web_and_matrix_config -- --nocapture` passed.
- `cargo test --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `pnpm --dir ui typecheck`, `pnpm --dir ui lint`, and `pnpm --dir ui build` passed.
- `pnpm --dir ui tauri build` passed and built:
  - `ui/src-tauri/target/release/ed-sentry-gui`
- Tauri build artifacts are locally buildable only in this milestone. CI packaging/upload was not added.

## Evidence

- `.omo/evidence/gui-webui-tauri/task-13-tauri-build.txt`
- `.omo/evidence/gui-webui-tauri/task-13-tauri-web-start.txt`
- `.omo/evidence/gui-webui-tauri/task-13-tauri-api-grep.txt`
- `.omo/evidence/gui-webui-tauri/task-13-code-review.md`
- `.omo/evidence/gui-webui-tauri/task-13-manual-qa-matrix.md`
- `.omo/evidence/gui-webui-tauri/task-13-cleanup-process-check.txt`
