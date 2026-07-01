# Dashboard external links

- The desktop GUI is hosted by the root Tauri app in `src/desktop_gui/mod.rs`, not by the tiny Windows launcher crate under `ui/src-tauri/`.
- Dashboard anchors that leave the app should use `target="_blank"` and `rel="noopener noreferrer"` for the browser/WebUI surface.
- In Tauri, `ui/src/components/dashboard/external-link.ts` intercepts anchor clicks when `__TAURI_INTERNALS__` exists and invokes the Rust `open_external_url` command.
- `open_external_url` is registered in `src/desktop_gui/mod.rs`, uses `tauri-plugin-opener`, and accepts only `http`/`https` URLs before opening the system browser.
- Current consumers are the Service Nodes Web Interface link, Tunnel public URL link, and ABOUT author/repository links.
- Manual QA on mock Vite surface (`VITE_DASHBOARD_ADAPTER=mock`) confirmed:
  - Dashboard nav button is visible.
  - Web Interface link opens a new page for `http://localhost:8765/` instead of navigating the dashboard tab.
  - ABOUT author opens `https://inara.cz/elite/cmdr/78197/` in a new page.
  - ABOUT repository opens `https://github.com/ControlNet/ed-sentry` in a new page.
