# Tunnel Web Policy Task 6 Notes

- The Web server owns a shared `WebTunnelState`, which wraps `TunnelLifecycleManager` plus per-run `TunnelAuth`. Runtime startup reuses the same handle through `WebServer::tunnel()` so HTTP routes and startup auto-start see the same active tunnel session.
- Host validation lives in `src/web/policy.rs` plus `src/web/policy/host.rs`. It accepts loopback/bind hosts, the exact active running tunnel host, and non-trycloudflare wildcard-bind hosts. It rejects arbitrary `trycloudflare.com` names unless they match `active_tunnel()`.
- `/api/config` is the only tunnel-bearer-protected route. Local loopback remains unchanged. Tunnel requests with an empty config password bypass auth; non-empty passwords require a valid bearer token tied to the active host and session.
- Tests use fake `cloudflared` shell scripts via `ED_SENTRY_CLOUDFLARED_PATH`; no real Cloudflare/network tunnel is used. Keep token values parsed in memory only and do not print them in test output or evidence.
