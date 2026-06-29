# Tunnel QA Task 11 Notes

- Deterministic tunnel integration tests can use the same fake `cloudflared` script pattern as earlier tasks: emit `https://fixture.trycloudflare.com`, then stay alive with `while :; do sleep 1; done`.
- `tokio_tungstenite::IntoClientRequest` allows replacing the `Host` header while connecting to `ws://127.0.0.1:<port>/ws`, which verifies active tunnel host policy locally without Cloudflare or external network.
- `pnpm --dir ui test:e2e` runs the mock-adapter browser suite against a local Vite preview and is sufficient for tunnel UI fixture QA; no live tunnel is needed.
