# Tunnel service options research - 2026-06-28

User hard requirements:

- End user clicks and uses it; no manual setup.
- No registration/account for the user.
- Free to use.
- Cross-platform.
- Windows users must not install extra software separately.
- Must support HTTP and WebSocket connections.

Findings:

- Cloudflare Quick Tunnel / TryCloudflare is the closest third-party fit.
  - Official docs: `cloudflared tunnel --url http://localhost:8080` creates a random `trycloudflare.com` public URL without Cloudflare account or DNS setup.
  - Official docs say Quick Tunnels are free, instant, no account/config, but intended for testing/development only, no SLA/uptime guarantee, 200 concurrent in-flight request limit, no SSE support.
  - Official Cloudflare Tunnel FAQ says WebSockets are fully supported.
  - Product integration path: bundle `cloudflared` per target platform inside release artifacts and spawn it as a child process; parse stdout/logs for the generated URL. End user installs nothing separately.
  - Risk: Cloudflare terms/license acceptance, dev/test-only positioning, no SLA, dependency on external binary/process and its output format.
- localhost.run fits no signup/free/SSH-based/no download, but is less suitable for this app.
  - It requires an SSH client and interactive process handling; while Windows often has OpenSSH, relying on system SSH violates the “no special install” spirit and is less controlled than bundling one known tunnel binary.
  - WebSocket support is not as clearly documented as Cloudflare’s official FAQ. Good for developer CLI, not ideal for packaged GUI feature.
- localtunnel is free/no signup and has an API, but is Node.js based.
  - End user would need Node/npm unless we embed a separate implementation or port/protocol client.
  - Public service reliability is a common concern; less robust for product feature.
- There is a Rust localtunnel implementation: `kaichaosun/rlt`, published as `localtunnel`, `localtunnel-client`, and `localtunnel-server` crates.
  - `localtunnel-client` exposes a Rust library API: `open_tunnel(ClientConfig) -> Result<String>` and returns the public URL.
  - Its README/docs state: "Known issue: the public proxy server is down, please setup your own server." Therefore it is not currently a reliable no-account public SaaS provider by itself.
  - The client implementation registers with the server and then proxies raw TCP with `tokio::io::copy_bidirectional`, so WebSocket passthrough is plausible if the server side supports upgrade/routing, but this must be verified with the project `/ws` endpoint before choosing it.
- Rust tunnel/provider options found:
  - `ferrotunnel`: Rust library-first reverse tunnel; embeddable client/server builder APIs; supports HTTP, TCP, WebSocket, gRPC; requires operating a server/token, not a no-registration public provider.
  - `bore`: simple Rust TCP tunnel with public `bore.pub`; raw TCP forwarding should carry WebSocket, but exposes a host:port rather than HTTPS subdomain and does not provide a browser-friendly HTTPS URL without extra reverse proxy/domain work.
  - `rustunnel`: Rust ngrok-like service, WebSocket control-plane and HTTP/TCP proxying; managed service appears to require token/account for real use; self-host possible.
  - `tinytun`: Rust HTTP/TCP tunnel with hosted default and self-host; needs deeper verification for WebSocket support before considering.
  - `burrow`: Rust embeddable client/server but explicitly says no WebSocket passthrough, so it fails this project requirement.
- ngrok, Tailscale Funnel, LocalXpose, rustunnel managed, etc. fail no-registration/no-account requirements or have paid/plan constraints.
- self-hosted tools such as bore, rathole, frp, chisel can be free and embeddable, but require the project owner to operate a public relay/domain; if used, the feature is no longer “third-party no-op” but can become a project-operated tunnel service.

Recommendation:

- User decision: Phase 1 should only implement Cloudflare Quick Tunnel.
- Phase 1 must still introduce a generic tunnel layer/provider abstraction so future providers can be added without rewriting GUI/status/security integration.
- Future provider examples include a self-hosted/own-server tunnel, including SSH-based providers.
- For fastest MVP: use bundled Cloudflare `cloudflared` Quick Tunnel as the first provider, with clear UI wording that tunnel availability is best-effort/free preview.
- For long-term/product reliability: design a provider trait and keep room for a project-operated relay later. Do not hard-code Cloudflare assumptions throughout the app.

Implementation notes for later plan:

- Add a provider abstraction: `TunnelProvider::start(local_url) -> TunnelSession { public_url, child/process/session handle }`.
- For Cloudflare provider, spawn bundled `cloudflared tunnel --url http://127.0.0.1:<port>`.
- Parse public URL from stdout/stderr lines containing `trycloudflare.com`.
- Check WebSocket path `/ws` through the tunnel during QA.
- Package Windows artifacts with `cloudflared.exe` if license/redistribution review passes; place it inside a package subfolder rather than the package root.
