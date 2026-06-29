# Tunnel Frontend Adapter Auth - Task 8

Date: 2026-06-28

## Durable Notes

- Frontend tunnel status parsing lives in `ui/src/adapters/tunnel.ts`; `ui/src/adapters/types.ts` re-exports the tunnel schema/types so existing adapter imports stay stable.
- Web adapter auth state lives only in browser `sessionStorage` key `ed-sentry:tunnel-auth-token`. The stored JSON includes the bearer value, parsed public host, and active tunnel session id so future config requests can clear stale auth before sending headers.
- Web config GET/PUT calls `GET /api/tunnel/status` without Authorization before attaching a stored bearer. If the running public host or session id differs, the token is removed and no bearer header is sent.
- `GET /api/tunnel/status`, `POST /api/tunnel/start`, `POST /api/tunnel/login`, snapshots, and WebSocket/events remain unauthenticated in the Web adapter.
- Tauri adapter exposes native `loadTunnelStatus()` and `startTunnel()` only; it does not read or write browser token storage and does not expose tunnel login.
- Adapter boundary tests are split by concern to stay below the 250 pure-LOC ceiling: core schema/store in `adapter-boundary.spec.ts`, Web tunnel auth in `tunnel-adapter-boundary.spec.ts`, and Tauri stream boundary in `tauri-adapter-boundary.spec.ts`.
