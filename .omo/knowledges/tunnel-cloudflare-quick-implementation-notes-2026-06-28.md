# Cloudflare Quick Tunnel implementation notes - 2026-06-28

## Implemented scope

- Phase 1 supports the top-level `[tunnel]` table with these safe defaults:

```toml
[tunnel]
provider = "cloudflare_quick"
auto_start = false
config_password = ""
```

- `provider = "cloudflare_quick"` is the only startable tunnel provider. The future SSH shape is inert and reports unsupported instead of spawning SSH.
- Cloudflare Quick Tunnel starts `cloudflared tunnel --url http://127.0.0.1:<bound_web_port>` and parses the generated `https://*.trycloudflare.com` URL from stdout or stderr.
- Windows GNU packages stage the pinned Cloudflare binary at `ed-sentry/tools/cloudflared/cloudflared.exe` and the Apache-2.0 license at `ed-sentry/tools/cloudflared/LICENSE-cloudflared.txt`.
- The package script pins Cloudflare release `2026.6.1` through `scripts/cloudflared-windows-amd64.url` and `scripts/cloudflared-windows-amd64.sha256`, caches the verified binary under `target/cloudflared-cache/`, and prints a SHA-256 line for the staged executable.

## Runtime behavior

- `auto_start = false` leaves the tunnel in the Service Nodes `TUNNEL` row as `START` when WebUI is bound and startable.
- Manual START is available from the WebUI and desktop Service Nodes surface. It is idempotent while a tunnel is starting or running.
- `auto_start = true` starts only after WebUI binds in watch-capable CLI or desktop runtimes.
- Replay mode, disabled WebUI, failed WebUI bind, and missing bound port do not start a tunnel.
- Tunnel failures do not stop the app or WebUI. Status becomes retryable error where applicable.

## Auth and access facts

- Local loopback WebUI behavior stays unchanged and does not require tunnel login.
- Desktop/Tauri uses native commands for tunnel status/start and config save. It does not use browser Bearer tokens or tunnel login.
- Empty `config_password` intentionally allows unauthenticated remote tunnel access to `SYSTEMS` and tunnel-host `GET /api/config` plus `PUT /api/config`.
- A non-empty `config_password` requires tunnel login for remote tunnel config access. The Web adapter stores the token only in `sessionStorage` under `ed-sentry:tunnel-auth-token`.
- Bearer tokens are tied to the active public host and active tunnel session id. They expire after 12 hours and become invalid after app restart or tunnel replacement.
- `/ws`, `/api/events`, snapshots, and tunnel status/start/login stay available through the active tunnel host without Bearer auth.

## Documentation and UI constraints

- Docs can describe Cloudflare Quick Tunnel as a convenient remote access path with external service limits. React UI source must not include no-SLA, dev-only, development-only, or similar Cloudflare limitation copy.
- Keep config examples secret-safe. Use `config_password = ""` or obvious placeholders that start with `<` if a placeholder is unavoidable.
- Do not document or commit real Matrix tokens, tunnel passwords, Cloudflare credentials, raw Journal paths, private home paths, or private keys.

## Verification commands for future edits

```bash
rg -n --hidden --glob '!target/**' --glob '!ui/node_modules/**' --glob '!ui/dist/**' --glob '!dist/**' 'access_token\s*=\s*"[^"<][^"]{8,}"|config_password\s*=\s*"[^"<][^"]{8,}"|BEGIN (RSA|OPENSSH|PRIVATE) KEY' README.md config.example.toml src ui .omo/knowledges
rg -n "no SLA|best-effort|best effort|development only|dev only|Cloudflare.*SLA" ui/src
python ~/.config/opencode/skills/secret-guard/scripts/scan_secrets.py tracked
python ~/.config/opencode/skills/secret-guard/scripts/scan_secrets.py gitignore
```

Expected results:

- The plan secret scan should have no matches except reviewed safe placeholders or the known pre-existing Rust fixture password finding when using Secret Guard tracked mode.
- The UI disclaimer grep should exit `1` with no matches.
- Secret Guard tracked mode currently reports the known fixture false positive at `src/app/config.rs:151`; do not add new findings.
- Secret Guard gitignore mode should report that common sensitive patterns are covered.

## Safe future tasks

- If a production relay or alternate provider is added later, keep it behind the existing provider/status/auth boundaries and add docs that distinguish provider guarantees without placing limitation copy in React UI source.
- If the Windows package layout changes, update README release artifacts and keep the license file beside any bundled third-party executable.
- If new SYSTEMS/config endpoints are added, protect them under tunnel host plus non-empty `config_password` unless they are explicitly listed as public.
