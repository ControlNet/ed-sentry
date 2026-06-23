# Single-exe GUI launcher PoC

Implemented on branch `poc/single-exe-gui-launcher`.

The Windows GNU package now builds one real `ed-sentry.exe` with desktop GUI support compiled by default. There is no `desktop-gui` Cargo feature gate: the root binary always includes `src/desktop_gui/`, pre-parses `--gui`, preserves normal CLI behavior when `--gui` is absent, and runs the Tauri desktop runner with `ui/src-tauri/tauri.conf.json` when `--gui` is present.

The root `tauri` dependency intentionally uses `default-features = false` with explicit `custom-protocol` and `wry` features. This keeps the GUI compiled by default while avoiding Tauri's default `common-controls-v6` feature, which caused the Windows GNU `ed-sentry.exe` import table to statically import `TaskDialogIndirect` from `comctl32.dll` and fail to load on older Windows environments before Rust `main` could run.

`ui/src-tauri` is now a tiny launcher-only package. Its `ed-sentry-gui.exe` resolves sibling `ed-sentry.exe`, starts it with `--gui`, and on Windows sets `CREATE_NO_WINDOW` so the console-subsystem child does not create a console window. The launcher crate no longer depends on Tauri or the root `ed-sentry` crate.

The root `build.rs` must run `tauri_build::build()` from `ui/src-tauri`. This is required even though `src/desktop_gui/mod.rs` uses `tauri::generate_context!("ui/src-tauri/tauri.conf.json")`: Tauri v2's ACL/capability generation is driven by the build script. If root `build.rs` is empty after moving the Tauri runner into `ed-sentry.exe`, the GUI can render but `@tauri-apps/api/window` commands can be denied/no-op because root `ed-sentry.exe` did not generate `capabilities.json` with `core:window:allow-close`, `allow-minimize`, `allow-start-dragging`, and `allow-toggle-maximize`. Keep the Tauri window label explicit as `"main"` in `ui/src-tauri/tauri.conf.json` so it matches `ui/src-tauri/capabilities/default.json`.

Packaging uses `./scripts/package-windows-gnu.sh`: it builds WebUI assets, builds root `ed-sentry.exe` with the default feature set, builds the tiny launcher with `cargo build --manifest-path ui/src-tauri/Cargo.toml`, copies `WebView2Loader.dll` from the root target's `webview2-com-sys` output, and stages the same release layout under `dist/ed-sentry/`.

After the TaskDialogIndirect fix, verify the Windows package with `x86_64-w64-mingw32-objdump -p dist/ed-sentry/ed-sentry.exe | rg -n "TaskDialogIndirect"`. Expected signal: no matches. `comctl32.dll` may still appear for `DefSubclassProc`, `RemoveWindowSubclass`, and `SetWindowSubclass`; those are not the failing import.

Measured package shape after the PoC: `ed-sentry.exe` about 73M, `ed-sentry-gui.exe` about 1.2M, `WebView2Loader.dll` 160K, `webui/` 468K, total `dist/ed-sentry/` about 75M. `file` reports `ed-sentry.exe` as PE console and `ed-sentry-gui.exe` as PE GUI.
