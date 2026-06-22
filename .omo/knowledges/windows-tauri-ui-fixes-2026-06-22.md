# Windows Tauri UI Fixes - 2026-06-22

## Context

The Windows GUI/WebUI pass fixed five user-reported issues:

- Double-clicking `ed-sentry-gui.exe` opened a console window.
- Tauri used the native Windows window frame instead of an app-defined titlebar.
- `Telemetry > Service Nodes` used inconsistent status badge colors.
- Per-second `runtime_status` updates were polluting `Recent Alerts` and `Comms Feed`.
- The desktop GUI loaded config from Tauri's app config directory instead of the packaged exe sibling `config.toml`.
- Tauri custom titlebar buttons did not minimize, maximize, or close the window.
- The Tauri custom titlebar could not be dragged reliably outside the small marked regions.
- Some blank background blocks around the titlebar tab row still did not drag reliably, and an attempted transparent overlay made the usable drag area worse.
- `Missions` overflowed short desktop windows because the workspace had a fixed `34rem` minimum height.

## Decisions

- The Windows GUI binary uses the `windows_subsystem = "windows"` release attribute in `ui/src-tauri/src/main.rs`; `objdump` confirmed `Subsystem (Windows GUI)`.
- Tauri window decorations are disabled in `ui/src-tauri/tauri.conf.json`; app chrome is rendered in `DashboardShell` only when `adapter.mode === "tauri"`.
- Desktop config resolution now prefers `std::env::current_exe().parent()/config.toml`, falling back to the app config directory only when `current_exe` cannot be resolved.
- Runtime status snapshots still publish snapshots, but no longer create or broadcast `EventFeedItem` entries.
- Service node colors are centralized through `serviceStatusBadgeTone(ServiceStatusKind)` and exposed with `data-status-kind` for regression tests.
- Mock data no longer includes a `runtime_status` feed item.
- Tauri v2 window APIs require explicit capability permissions. The default desktop capability now includes `core:window:allow-close`, `core:window:allow-minimize`, `core:window:allow-toggle-maximize`, and `core:window:allow-start-dragging`.
- The custom titlebar starts drag from `onPointerDownCapture` on the header for non-interactive left-button pointer down events, while buttons, links, form controls, roles, and `[data-window-control]` remain interactive click targets.
- The transparent middle-nav overlay approach was removed. The titlebar now marks intended regions with `data-titlebar-drag-region` and `data-titlebar-no-drag`, leaving actual drag startup to the capture-phase handler so empty gaps and background blocks are handled consistently without covering tab buttons.
- The left brand block and right sync-status block now also apply `data-tauri-drag-region` directly to their non-interactive children (`brand-mark`, `brand-label`, `status-label`, and status dot), because Windows hit testing showed that relying only on the parent drag region left small non-draggable holes.
- `?debug_titlebar_drag=1` enables a visual hitmap: green striped regions are intended draggable titlebar background, and red striped regions are tab/window-control buttons that must remain clickable and non-draggable. The Playwright screenshot is `.omo/evidence/gui-webui-tauri/titlebar-drag-hitmap.png`.
- The titlebar hitmap is URL-only.
- The shell `main` region is a vertical flex container. Tactical workspaces flex into the remaining content height with `min-height: 0`, and `Missions` uses a two-column grid with internally scrollable panels rather than forcing page-level scrolling.

## Verification

- `cargo test -- --test-threads=1` passed.
- `cargo test --manifest-path ui/src-tauri/Cargo.toml` passed.
- `pnpm --dir ui lint` passed.
- `pnpm --dir ui typecheck` passed.
- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui exec playwright test e2e/tauri-window-chrome.spec.ts` passed.
- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui build` passed.
- `VITE_DASHBOARD_ADAPTER=mock pnpm --dir ui exec playwright test e2e/dashboard-smoke.spec.ts e2e/tauri-window-chrome.spec.ts e2e/reference-redesign.spec.ts` passed with 14 tests, including `@tauri-window-chrome visualizes titlebar drag hitmap`.
- `@missions workspace fits a short desktop viewport` reproduces the old overflow (`main.scrollHeight` exceeded `main.clientHeight`) and now passes at `1280x640`.
- `@tauri-window-chrome enables frameless drag and window controls` verifies disabled native decorations, required Tauri window permissions, drag startup, and window command calls.
- `./scripts/package-windows-gnu.sh` rebuilt `dist/ed-sentry-x86_64-pc-windows-gnu.zip`.

## Artifact

Latest package from this pass:

- `dist/ed-sentry-x86_64-pc-windows-gnu.zip`
- SHA256: `d1c05ca07a4da0d0ec31e4831017c560992905a8139759331a2bf19b57f133c8`
- GUI exe SHA256: `8a8ee51121e0022f1e59b78bea1f71eafac93b158104cde529657d0c9af38923`
- WebView2 loader SHA256: `8427b1fc58ec707813e5c0a51eb5d69397bb333250a7b891be4d3b123f1e0f1c`
- Contains `ed-sentry.exe`, `ed-sentry-gui.exe`, `WebView2Loader.dll`, `config.toml`, and `webui/index.html`.
