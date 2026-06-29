# Tunnel Tauri Commands Task 7 Notes

- Native desktop tunnel commands are `load_tunnel_status` and `start_tunnel` in `src/desktop_gui/mod.rs`.
- The commands use `DesktopRuntime::tunnel_status()` and `DesktopRuntime::start_tunnel()` directly, so desktop/Tauri local access does not use tunnel passwords, Bearer tokens, Web API routes, or browser storage.
- `save_config` remains a native desktop config-file write path via `AppConfig::write_update_to_source`; do not route desktop saves through the Web API.
- Frontend adapter methods are optional and named `loadTunnelStatus()` and `startTunnel()` on `DashboardAdapter`; default Tauri transport maps them to the native command names.
- Web adapter auth headers/sessionStorage remain out of scope for task 7 and are reserved for task 8.
