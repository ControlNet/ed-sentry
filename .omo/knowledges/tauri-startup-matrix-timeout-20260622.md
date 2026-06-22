# Tauri startup delay profiling - Matrix timeout

Date: 2026-06-22

## Finding

The Tauri GUI startup delay was caused by the desktop shell synchronously starting the full monitor runtime inside `tauri::Builder::setup`.

The measured blocking stage was Matrix startup:

- Dummy local config enabled Matrix with an unreachable homeserver.
- CLI runtime emitted `Warning: Matrix delivery disabled: Matrix startup timed out after 10s` at `10024 ms`.
- The Tauri shell previously called `tauri::async_runtime::block_on(DesktopRuntime::start(config.clone()))` from `ui/src-tauri/src/lib.rs` setup, so the same Matrix timeout could block GUI window initialization.

## Fix

`ui/src-tauri/src/lib.rs` now loads config during setup, manages shared `DesktopState`, and starts `DesktopRuntime::start` in a background Tauri async task via `spawn_desktop_runtime`.

`load_snapshot` waits on a `tokio::sync::watch` startup signal if the runtime is still starting. This keeps the window setup nonblocking while preserving the existing frontend command contract.

## Regression Guard

`ui/e2e/tauri-window-chrome.spec.ts` includes a source-boundary test:

- rejects `block_on(DesktopRuntime::start` in the Tauri shell source
- requires `spawn_desktop_runtime`

This test first failed against the blocking implementation, then passed after the fix.

## Verification Signals

- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui exec playwright test e2e/tauri-window-chrome.spec.ts -g "does not block"` passed.
- `cargo test --manifest-path ui/src-tauri/Cargo.toml` passed.
- `pnpm --dir ui lint` passed.
- `pnpm --dir ui typecheck` passed.
- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui build` passed.
- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui exec playwright test e2e/dashboard-smoke.spec.ts e2e/tauri-window-chrome.spec.ts e2e/reference-redesign.spec.ts` passed with 15 tests.
- `./scripts/package-windows-gnu.sh` rebuilt `dist/ed-sentry-x86_64-pc-windows-gnu.zip`.
