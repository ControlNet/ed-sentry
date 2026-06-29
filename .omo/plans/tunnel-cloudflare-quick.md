# tunnel-cloudflare-quick - Work Plan

## TL;DR (For humans)

**What you'll get:** A one-click tunnel in the existing WebUI/desktop experience: users can auto-start it from config or manually start it from the Service Nodes area, then open or scan the generated public link. If a tunnel password is configured, remote tunnel users must log in before using SYSTEMS/config; if it is empty, remote access is intentionally open.

**Why this approach:** Phase 1 keeps provider scope narrow by using Cloudflare Quick Tunnel only, but introduces a small provider layer so a future SSH/self-hosted provider can fit without reworking UI/auth/status. The implementation preserves the current local WebUI and desktop behavior while adding tunnel handling only at the shared Web runtime and explicitly allowed UI surfaces.

**What it will NOT do:** It will not implement SSH yet, not add localtunnel/bore/ferrotunnel providers, not hash the local TOML password, and not redesign or polish unrelated UI. It also will not show Cloudflare no-SLA/dev-only disclaimer text in the UI; any risk or limitation notes are docs-only.

**Effort:** Large
**Risk:** High - this crosses Rust config/runtime/Web policy, Tauri adapters, React UI, auth, WebSocket behavior, and Windows packaging with an external executable.
**Decisions to sanity-check:** `cloudflared.exe` is downloaded/pinned during packaging rather than committed, staged at `tools/cloudflared/cloudflared.exe`, and `config_password = ""` intentionally means unauthenticated tunnel access.

Your next move: either start work from this plan, or run a high-accuracy plan review first. Full execution detail follows below.

---

> TL;DR (machine): Large/high-risk feature: add `[tunnel]`, Cloudflare-only provider abstraction, tunnel auth/status/start APIs, strictly scoped UI, Windows cloudflared subfolder packaging, and full Rust/TS/package/WebSocket QA.

## Scope
### Must have
- Add a top-level TOML config table with defaults:

  ```toml
  [tunnel]
  provider = "cloudflare_quick"
  auto_start = false
  config_password = ""
  ```

- Missing `[tunnel]` must keep those defaults; wrong-typed tunnel keys must warn and keep defaults using existing config warning style.
- Add tunnel config to `AppConfig`, `RuntimeConfig`, config loading, config write, config API DTOs, Tauri config DTOs, frontend schemas, and SYSTEMS config UI.
- `config_password` is stored in plaintext TOML for this phase. If non-empty, config APIs/UI must avoid echoing it back as a readable value; use keep/replace/clear semantics similar to Matrix token handling. If empty/missing, tunnel auth is disabled.
- Define only two provider types conceptually: `CloudflareQuickTunnelProvider` and inert future `SshTunnelProvider` shape. Only Cloudflare can actually start in Phase 1.
- The concrete startable backend provider type must be named `CloudflareQuickTunnelProvider` or be an obviously equivalent struct with that public/provider label. The future SSH placeholder must be named `SshTunnelProvider` or be an obviously equivalent inert type. `provider = "ssh"` may parse but must start as `unsupported` and must not spawn SSH.
- Start Cloudflare Quick Tunnel by running bundled `cloudflared tunnel --url http://127.0.0.1:<bound_web_port>`.
- Parse the generated `https://*.trycloudflare.com` URL from `cloudflared` stdout/stderr and expose it in tunnel status.
- `auto_start = true` starts the tunnel only after WebUI successfully starts in watch-capable CLI or desktop runtime. `[web] enabled = false` and replay mode must not start the tunnel.
- Manual START must be available from the new TUNNEL Service Nodes UI row only when WebUI is bound and a local port exists. If `[web] enabled = false`, replay mode is active, bind failed, assets are missing, or no bound port exists, the TUNNEL row shows an unavailable/disabled state and start returns a non-spawning disabled/unavailable error. Start must be idempotent: repeated clicks while starting/running must not spawn multiple `cloudflared` processes.
- Tunnel failure must not crash or block WebUI/app startup; status becomes error/warning/retryable.
- Add exactly these Web API endpoints: `GET /api/tunnel/status`, `POST /api/tunnel/start`, and `POST /api/tunnel/login`. Tauri commands may be named idiomatically, but frontend adapter methods must map to these semantics exactly.
- Serialized tunnel status values are exactly: `disabled`, `start`, `starting`, `running`, `error`, and `unsupported`. `start` means ready/manual-startable, not already running.
- Tunnel-origin requests are identified only by the active running session's exact public host. Before the URL is parsed, while starting, after provider crash/stop, or after restart replaces the public host/session id, no stale tunnel host is trusted. Do not trust arbitrary `trycloudflare.com`/Forwarded headers and do not weaken loopback/unknown Host policy.
- Access/auth matrix:

  | Context | `config_password` | Behavior |
  | --- | --- | --- |
  | Local loopback WebUI | any | Existing behavior unchanged; no tunnel login required. |
  | Desktop/Tauri adapter | any | Existing behavior unchanged; no tunnel login required. |
  | Tunnel host | empty/missing | SYSTEMS/config page and related APIs are accessible without auth; user accepts risk. |
  | Tunnel host | non-empty | SYSTEMS/config page and related APIs require Bearer token from tunnel login. |
  | Tunnel host | non-empty + valid token | SYSTEMS/config access and config mutation are allowed. |
  | Tunnel host | non-empty + missing/invalid/expired token | Protected page/API returns auth-required/unauthorized state. |
  | Unknown Host | any | Rejected as today. |

- Use a maintained JWT-like Rust library (`jsonwebtoken`) for token creation/validation; no custom token signing. Tokens must include expiry, be signed with an app-generated per-run secret, be accepted via `Authorization: Bearer`, and reject wrong password, malformed tokens, invalid signatures, and expired tokens.
- Tunnel auth tokens must include the active tunnel host and active tunnel session id claims. A token is valid only when its host/session claims match the current running tunnel. Tokens expire after 12 hours. App restart invalidates all tokens via the per-run signing secret.
- Frontend Web mode stores the token only in `sessionStorage` under key `ed-sentry:tunnel-auth-token`; clear it on login failure, 401/403 from protected tunnel APIs, host/session mismatch, or browser session end. Tauri mode must not store or use tunnel auth tokens for native local commands.
- Preserve read-only dashboard/status/WebSocket access unless it is part of SYSTEMS/config; `/ws` and `/api/events` must work through Cloudflare tunnel.
- Protected current route inventory under tunnel host + non-empty password: `GET /api/config` and `PUT /api/config`. Policy rule for future additions: every endpoint used exclusively by SYSTEMS/config UI is protected unless explicitly listed as unprotected. Explicitly unprotected under tunnel: static assets, `GET /api/health`, `GET /api/snapshot`, `GET /api/web/status`, `GET /api/matrix/status`, `GET /api/web/policy`, `GET /api/events`, `GET /ws`, `GET /api/tunnel/status`, `POST /api/tunnel/start`, and `POST /api/tunnel/login`.
- Strict UI scope: only change (1) tunnel-mode SYSTEMS login prompt, (2) Service Nodes TUNNEL row/link/QR, and (3) SYSTEMS config fields for `[tunnel]`.
- Link behavior: running tunnel shows the public link, click opens it, and pointer hover plus keyboard focus render a QR code. Touch-only devices only need the click/tap link behavior for Phase 1.
- Do not display Cloudflare no-SLA/dev-only/best-effort disclaimer in React UI/status/QR/login/config surfaces.
- Windows GNU package must include Cloudflare binary at exactly `ed-sentry/tools/cloudflared/cloudflared.exe` inside the zip, not at package root. Package verification must include a SHA-256 line for that executable.
- Packaging pin source is exact: add committed text metadata files `scripts/cloudflared-windows-amd64.url` containing the pinned GitHub release asset URL and `scripts/cloudflared-windows-amd64.sha256` containing the expected SHA-256 for that asset. `scripts/package-windows-gnu.sh` downloads to `target/cloudflared-cache/cloudflared-windows-amd64.exe` if no verified cache exists, verifies SHA-256, reuses verified cache offline, and fails if neither network nor a valid cache is available.
- License decision is exact: because `cloudflared` is Apache-2.0, add a committed `third_party/cloudflared/LICENSE.txt` containing the upstream Apache-2.0 license text and stage it as `ed-sentry/tools/cloudflared/LICENSE-cloudflared.txt` next to the executable.
- Update README/config template/package docs enough for operation and license compliance, while keeping secrets/tokens out of committed docs.
- Documentation may mention empty-password risk and Cloudflare operational limitations. React UI/status/login/QR/config/service-node copy must not mention Cloudflare no-SLA/dev-only/best-effort disclaimers.
- Because frontend/Tauri/packaging assets change, final verification must rebuild Windows artifact with `./scripts/package-windows-gnu.sh`.

### Must NOT have (guardrails, anti-slop, scope boundaries)
- Must not implement SSH tunnel execution in Phase 1; `SshTunnelProvider` may be a type/config shape only and must return explicit unsupported state if selected/startable.
- Must not add localtunnel, bore, ferrotunnel, rustunnel, ngrok, Tailscale, or other providers.
- Must not add a generic plugin/provider registry beyond the minimum trait/enum/session status needed for Cloudflare now and SSH later.
- Must not modify unrelated UI content, layout, styling, text, routes, or components beyond the three allowed UI surfaces.
- Must not modify `reference-design/design1.tsx`.
- Must not require users to install cloudflared, Node.js, SSH, or any separate tunnel software on Windows.
- Must not commit real secrets, tokens, passwords, Cloudflare credentials, raw Journal data, or private paths.
- Must not expose existing non-empty `config_password` back to the browser as plaintext through config APIs.
- Must not make authenticated tunnel mode read-only; after auth, config mutation is allowed.
- Must not start WebUI/tunnel in replay mode.
- Must not let stale/old tunnel hosts remain trusted after tunnel stop, crash, or replacement.
- Must not use `git clean`.

## Verification strategy
> Zero human intervention - all verification is agent-executed.
- Test decision: tests-after for each implementation slice using existing Rust `cargo test`, TypeScript `pnpm --dir ui build`/typecheck/e2e infrastructure, targeted unit/integration tests for new tunnel/auth policy, and final Windows package rebuild.
- Evidence: each todo writes command output or a short verification note under `.omo/evidence/tunnel-cloudflare-quick/task-<N>.md` (or `.log` for long command output). Final verification writes `.omo/evidence/tunnel-cloudflare-quick/final-*.md`.
- “Zero human intervention” means every verification command and assertion is agent-executed. The final verification wave still reports results and waits for the user before declaring the whole implementation complete; that wait is a handoff/acceptance gate, not a manual testing requirement.
- Required broad commands before final handoff:

  ```bash
  cargo fmt --check
  cargo test --all
  cargo clippy --all-targets --all-features -- -D warnings
  pnpm --dir ui build
  pnpm --dir ui lint
  ./scripts/package-windows-gnu.sh
  unzip -l dist/ed-sentry-x86_64-pc-windows-gnu.zip | rg 'ed-sentry/tools/cloudflared/cloudflared\.exe$'
  rg -n "no SLA|best-effort|best effort|development only|dev only|Cloudflare.*SLA" ui/src
  ```

- Expected broad signals: Rust and UI commands exit 0; package script regenerates `dist/ed-sentry-x86_64-pc-windows-gnu.zip` and prints SHA-256 lines for existing required artifacts plus `tools/cloudflared/cloudflared.exe`; `unzip|rg` finds the exact cloudflared path; disclaimer `rg` exits 1 with no UI matches.

## Execution strategy
### Parallel execution waves
> Target 5-8 todos per wave. Fewer than 3 (except the final) means you under-split.

- Wave 1: Backend data foundations and tests (`[tunnel]` config model, API DTOs, status/auth/provider types) can proceed in parallel after checking compile dependencies.
- Wave 2: Runtime/API behavior (Cloudflare provider, shared lifecycle, Web policy/auth routes, Tauri adapter commands) builds on Wave 1.
- Wave 3: Strictly scoped UI and adapter integration builds on Wave 2 DTO/API shape.
- Wave 4: Packaging/docs/QA hardening and full verification after functional work compiles.

### Allowed frontend file categories
- Allowed React surface components: `ui/src/components/dashboard/service-status-panel.tsx`, `ui/src/components/dashboard/config-panel.tsx`, `ui/src/components/dashboard/config-core-sections.tsx`, and a new narrowly named tunnel auth/QR component colocated under `ui/src/components/dashboard/` if needed.
- Allowed frontend plumbing: `ui/src/adapters/config.ts`, `ui/src/adapters/types.ts`, `ui/src/adapters/web.ts`, `ui/src/adapters/tauri.ts`, `ui/src/store/dashboard-store.ts`, and test/fixture files required to validate those changes.
- Forbidden without explicit user approval: unrelated dashboard pages, shell/navigation copy/layout, global styling/theme files, non-tunnel component rewrites, and visual polish outside the allowed surfaces.

### Dependency matrix
| Todo | Depends on | Blocks | Can parallelize with |
| --- | --- | --- | --- |
| 1 config model | none | 2, 3, 4, 5, 8 | 2 after type sketch only, 12 docs draft |
| 2 status/provider contracts | none | 3, 4, 5, 6, 8, 9 | 1 |
| 3 config API/write DTOs | 1 | 5, 8, 9 | 4 |
| 4 JWT/auth policy primitives | 1, 2 | 5, 6, 9 | 3 |
| 5 Cloudflare provider/lifecycle | 1, 2, 4 | 6, 8, 10 | 7 after API contracts |
| 6 Web API host/auth routes | 3, 4, 5 | 8, 9, final QA | 7 |
| 7 Tauri adapter commands | 2, 3 | 8, 9 | 6 |
| 8 UI adapters/schemas | 3, 5, 6, 7 | 9 | none |
| 9 scoped UI surfaces | 8 | 11, final QA | 10 |
| 10 packaging cloudflared | 5 | 11, final package | 9 |
| 11 WebSocket/e2e/package QA | 6, 9, 10 | final verification | 12 |
| 12 docs/knowledge | 1, 10 | final verification | 11 |

## Todos
> Implementation + Test = ONE todo. Never separate.
<!-- APPEND TASK BATCHES BELOW THIS LINE WITH edit/apply_patch - never rewrite the headers above. -->
- [x] 1. Add `[tunnel]` Rust config model, defaults, parser, and unit tests
  What to do / Must NOT do: Add a `TunnelConfig` model with `provider`, `auto_start`, and `config_password` to the config layer and runtime config. Default to `provider = "cloudflare_quick"`, `auto_start = false`, `config_password = ""`. Implement TOML reading for top-level `[tunnel]` using existing typed-read warning style. Ensure wrong typed keys warn/default and missing table defaults. Do not add SSH runtime behavior; if a provider enum includes `ssh`, treat it as future/inert. Do not expose password in debug output; follow Matrix token redaction precedent where applicable.
  Parallelization: Wave 1 | Blocked by: none | Blocks: 3, 5, 8
  References (executor has NO interview context - be exhaustive): `src/config/model.rs:5-37`; `src/config/web.rs:5-43`; `src/config/matrix.rs:50-68,172-225`; `src/config/runtime.rs:8-35`; `.omo/knowledges/tunnel-ui-requirements-2026-06-28.md:5-28`.
  Acceptance criteria (agent-executable): `cargo test --all config_tunnel` exits 0 and covers missing defaults, explicit values, wrong types warnings/defaults, and Debug/redaction behavior for non-empty password.
  QA scenarios (name the exact tool + invocation): Happy: `cargo test --all config_tunnel_defaults config_tunnel_enabled_values` records defaults/explicit parsing in `.omo/evidence/tunnel-cloudflare-quick/task-1.md`. Failure: `cargo test --all config_tunnel_wrong_typed_keys_warn` records warnings and default retention.
  Final verification: after this todo, run `cargo test --all config_tunnel` and append pass/fail plus warnings count assertions to `.omo/evidence/tunnel-cloudflare-quick/task-1.md`.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 2. Define tunnel status/session/provider contracts without over-generalizing providers
  What to do / Must NOT do: Add backend structs/enums for tunnel provider (`cloudflare_quick`, future `ssh`), tunnel state (`disabled`, `start`, `starting`, `running`, `warning/error` or equivalent), public URL, provider label, checked timestamp, and retryable error. Add a small provider trait/manager API sufficient for Cloudflare start/stop/status and future SSH slot. Do not create plugin registries, provider discovery, localtunnel/bore/etc. abstractions, or SSH execution. Integrate tunnel status into snapshots/config status views separately from `WebStartupStatus`; do not overload `web.url`.
  Parallelization: Wave 1 | Blocked by: none | Blocks: 5, 6, 7, 8, 9
  References: `src/app/status.rs:24-52,104-180`; `src/app/snapshot.rs:14-47`; `src/app/runtime/service.rs:43-86`; `.omo/knowledges/tunnel-service-options-2026-06-28.md:39-53`.
  Acceptance criteria: `cargo test --all tunnel_status` exits 0 and verifies disabled/manual-start/starting/running/error serialization plus future SSH unsupported state.
  QA scenarios: Happy: serialize a running Cloudflare status with `https://fixture.trycloudflare.com` and assert public URL present. Failure: select/start future SSH and assert explicit unsupported status, not process spawn. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-2.md`.
  Final verification: after this todo, run `cargo test --all tunnel_status` and verify serialized values are exactly `disabled`, `start`, `starting`, `running`, `error`, `unsupported`.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 3. Extend config API DTOs, TOML writer, and frontend schemas for tunnel config without echoing passwords
  What to do / Must NOT do: Extend `EditableConfigView`, `EditableConfigUpdate`, Rust config edit types, web config API, desktop config API, and `ui/src/adapters/config.ts` schemas/types with tunnel config. For `config_password`, do not return the existing secret plaintext when non-empty; expose a presence flag and use keep/replace/clear semantics similar to Matrix token replacement. Write `[tunnel]` TOML updates with `toml_edit` while preserving unrelated fields. Do not break autosave for non-protected fields.
  Parallelization: Wave 1/2 | Blocked by: 1 | Blocks: 6, 8, 9
  References: `src/app/config.rs:1-21,107-121`; `src/app/config/types.rs:17-24,184-191`; `src/config/write.rs:77-115`; `src/config/write/apply.rs:14-29,98-119`; `ui/src/adapters/config.ts:69-149`; `ui/src/components/dashboard/config-form-model.ts:22-106`; `src/desktop_gui/mod.rs:31-58`.
  Acceptance criteria: `cargo test --all config_write tunnel_config_api` and `pnpm --dir ui build` exit 0; tests prove non-empty `config_password` is not echoed, replacement writes a new value, clear removes/empties it, and unrelated Web/Matrix fields are preserved.
  QA scenarios: Happy: write update with `auto_start = true` and password replacement, reload, assert TOML contains expected `[tunnel]`. Failure: config API view for non-empty password returns only `config_password_present = true` (or equivalent), not plaintext. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-3.md`.
  Final verification: after this todo, run `cargo test --all config_write` and `cargo test --all tunnel_config_api`, then `pnpm --dir ui build`; append all pass signals to task evidence.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 4. Add JWT-like tunnel auth primitives using `jsonwebtoken`
  What to do / Must NOT do: Add `jsonwebtoken` with an explicit crypto backend compatible with the repo toolchain. Implement tunnel auth claims with subject/purpose, issued/expiry timestamp, expiry timestamp, active tunnel host, and active tunnel session id. Sign with app-generated per-run secret; do not store tokens on disk. Validate only `Authorization: Bearer <token>` for protected Web APIs. Wrong password must not issue a token. Empty `config_password` bypasses tunnel auth. Token lifetime is exactly 12 hours. Token must be invalid when app restarts, active tunnel host changes, active tunnel session id changes, token expires, token is malformed, or signature is invalid. Do not hand-roll signing, do not add accounts/roles/refresh tokens/OAuth/database/persistent sessions.
  Parallelization: Wave 1/2 | Blocked by: 1, 2 | Blocks: 6, 9
  References: `Cargo.toml:13-28`; `src/web/policy.rs:154-170,206-239`; `ui/src/adapters/tauri.ts:184-192`; user requirement in `.omo/knowledges/tunnel-ui-requirements-2026-06-28.md:21-24`.
  Acceptance criteria: `cargo test --all tunnel_auth` exits 0 and covers correct password -> token, wrong password -> 401/403/no token, invalid/malformed/expired token rejected, empty password bypasses protection.
  QA scenarios: Happy: issue token with correct password and validate Bearer header. Failure: expired token and wrong signature reject protected action. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-4.md`.
  Final verification: after this todo, run `cargo test --all tunnel_auth` and confirm evidence includes correct, wrong-password, malformed, expired, stale-host, and stale-session cases.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 5. Implement Cloudflare Quick Tunnel provider process handling and shared lifecycle manager
  What to do / Must NOT do: Implement concrete `CloudflareQuickTunnelProvider` to resolve the packaged/dev cloudflared binary path, spawn `cloudflared tunnel --url http://127.0.0.1:<bound_web_port>`, read stdout/stderr asynchronously, parse the first `https://*.trycloudflare.com` URL, track child lifetime, and terminate/cleanup on drop. Add timeout/error handling and retryable status. Start automatically only when `[web] enabled = true`, WebUI binds successfully, and `[tunnel] auto_start = true`. Manual start must be idempotent and not spawn duplicates; when no bound Web port exists it returns disabled/unavailable without spawning. On stop/crash/restart, clear active tunnel host/session so stale hosts are rejected. Do not show Cloudflare disclaimers in status/UI text. Do not start tunnel for replay or when WebUI disabled.
  Parallelization: Wave 2 | Blocked by: 1, 2, 4 | Blocks: 6, 7, 8, 10, 11
  References: `src/web/server.rs:72-132`; `src/app/runtime/web.rs:8-19`; `src/app/runtime/desktop.rs:53-64`; Cloudflare docs evidence in `.omo/knowledges/tunnel-service-options-2026-06-28.md:14-19,49-52`; Metis constraint exact path `tools/cloudflared/cloudflared.exe`.
  Acceptance criteria: `cargo test --all tunnel_provider tunnel_lifecycle` exits 0 and covers URL parser, stdout/stderr handling, no URL timeout, child failure, drop cleanup, auto-start gating, and idempotent manual start.
  QA scenarios: Happy: fake cloudflared process/script emits `https://fixture.trycloudflare.com` and manager enters running with that URL. Failure: fake process exits before URL and manager reports error without affecting Web status. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-5.md`.
  Final verification: after this todo, run `cargo test --all tunnel_provider` and `cargo test --all tunnel_lifecycle`; evidence must include idempotent start, no-bound-port no-spawn, and stale-host clear on crash/restart.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 6. Add Web API routes and policy matrix for tunnel status/start/login/protected config access
  What to do / Must NOT do: Extend Axum router with exactly `GET /api/tunnel/status`, `POST /api/tunnel/start`, and `POST /api/tunnel/login`. Modify Host validation to trust only the active running tunnel public host, not arbitrary `trycloudflare.com`, and reject stale hosts after stop/crash/restart. Apply auth matrix: tunnel host + non-empty password protects current SYSTEMS/config APIs `GET /api/config` and `PUT /api/config`; future endpoints used exclusively by SYSTEMS/config UI are protected unless explicitly listed as unprotected. Valid Bearer token permits reads/writes; empty password allows access. Keep local loopback behavior unchanged and unknown Host rejected. Explicitly keep unprotected under tunnel: static assets, `GET /api/health`, `GET /api/snapshot`, `GET /api/web/status`, `GET /api/matrix/status`, `GET /api/web/policy`, `GET /api/events`, `GET /ws`, `GET /api/tunnel/status`, `POST /api/tunnel/start`, `POST /api/tunnel/login`. Preserve `/ws`/`/api/events` through tunnel. Do not make tunnel mode read-only after auth.
  Parallelization: Wave 2 | Blocked by: 3, 4, 5 | Blocks: 8, 9, 11
  References: `src/web/policy.rs:83-104,115-170,206-239`; `src/web/policy/config_api.rs:13-46`; `src/web/policy/ws.rs:38-45`; access/auth matrix in this plan at `.omo/plans/tunnel-cloudflare-quick.md:45-61`; user tunnel auth requirements in `.omo/knowledges/tunnel-ui-requirements-2026-06-28.md:16-28`.
  Acceptance criteria: `cargo test --all web_policy tunnel_auth config_api tunnel_routes` exits 0 and covers active tunnel host accepted, unrelated tunnel-like host rejected, empty password unauthenticated config GET/PUT succeeds, non-empty password unauthenticated config GET/PUT fails, login succeeds/fails correctly, valid token allows config mutation, local loopback unchanged.
  QA scenarios: Happy: request to active tunnel Host with valid token saves config. Failure: request to `evil.trycloudflare.com` or missing token returns rejected/unauthorized. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-6.md`.
  Final verification: after this todo, run `cargo test --all web_policy`, `cargo test --all tunnel_routes`, and `cargo test --all config_api`; evidence must include exact endpoint names and protected/unprotected route matrix.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 7. Add Tauri tunnel commands and adapter method contracts without changing local desktop behavior
  What to do / Must NOT do: Add Tauri commands for tunnel status/start and any local config/tunnel state needed by the Service Nodes TUNNEL row in desktop mode. Extend `DashboardAdapter`/Tauri transport types with optional tunnel methods while preserving Web adapter and existing `load_snapshot`, `load_config`, `save_config` behavior. Desktop local access must not require tunnel password; password auth applies to remote tunnel Web access, not native Tauri commands. Do not route desktop config saves through Web API.
  Parallelization: Wave 2 | Blocked by: 2, 3, 5 | Blocks: 8, 9
  References: `src/desktop_gui/mod.rs:15-58,60-85`; `src/desktop_gui/state.rs` (read before editing); `ui/src/adapters/types.ts:214-220`; `ui/src/adapters/tauri.ts:18-24,37-97`; `src/app/runtime/desktop.rs:46-97`.
  Acceptance criteria: `cargo test --all desktop_gui tunnel` and `pnpm --dir ui build` exit 0; mocked Tauri adapter can load status/start tunnel and existing config save tests remain passing.
  QA scenarios: Happy: desktop adapter start command transitions tunnel status to starting/running using mocked manager. Failure: start command returns retryable error when provider unavailable without breaking `load_snapshot`. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-7.md`.
  Final verification: after this todo, run `cargo test --all desktop_gui` and `cargo test --all tunnel`; evidence must show existing `load_snapshot/load_config/save_config` behavior remains passing.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 8. Extend frontend adapters/schemas/stores for tunnel status/start/login and auth headers
  What to do / Must NOT do: Update frontend Zod schemas and TypeScript types for tunnel config/status with exact serialized status values `disabled`, `start`, `starting`, `running`, `error`, `unsupported`. Add adapter methods for tunnel status/start/login in both Web and Tauri modes. Web adapter stores tunnel auth token only in `sessionStorage` key `ed-sentry:tunnel-auth-token` and sends `Authorization: Bearer` only for protected tunnel requests after login. Clear that key on login failure, protected-route 401/403, tunnel host/session change, and browser session end. Ensure config form update supports password keep/replace/clear. Do not add UI surfaces yet except minimal type-safe plumbing. Do not leak password through logs/errors; reuse existing redaction patterns.
  Parallelization: Wave 3 | Blocked by: 3, 5, 6, 7 | Blocks: 9
  References: `ui/src/adapters/config.ts:69-149`; `ui/src/adapters/types.ts:214-220,233-251`; `ui/src/adapters/web.ts:47-97,173-177`; `ui/src/adapters/tauri.ts:18-24,37-97,184-192`; `ui/src/store/dashboard-store.ts:36-91`.
  Acceptance criteria: `pnpm --dir ui build` and `pnpm --dir ui lint` exit 0; unit/e2e adapter fixture updates parse tunnel status/config and reject malformed payloads.
  QA scenarios: Happy: mocked Web adapter login stores token and protected config request includes Bearer header. Failure: malformed tunnel status payload fails Zod parse with adapter error. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-8.md`.
  Final verification: after this todo, run `pnpm --dir ui build` and `pnpm --dir ui lint`; evidence must include token storage key and clear-on-auth-failure fixture assertions.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 9. Implement only the approved UI surfaces: Service Nodes TUNNEL, SYSTEMS auth prompt, and tunnel config fields
  What to do / Must NOT do: Add TUNNEL to `TELEMETRY, SERVICE NODES` beside existing Matrix/Web service nodes with UI labels derived from exact tunnel statuses: `START` for `start`, `STARTING` for `starting`, `RUNNING` for `running`, `ERROR` for `error`, `UNSUPPORTED` for `unsupported`, and disabled/unavailable copy for `disabled`. START clicks adapter start only when status is `start` or `error` retryable. RUNNING shows public link; click opens URL; pointer hover and keyboard focus show QR code; touch-only only needs tap/click link behavior. Add SYSTEMS tunnel login prompt only for tunnel access when `config_password` is non-empty and token missing/invalid; after login, show normal SYSTEMS/config and allow saves. Add `[tunnel]` config fields to SYSTEMS/config UI with provider (Cloudflare now, SSH future/unsupported if shown), auto_start, and config_password replacement/clear. Do not modify unrelated UI content/layout/style/text. Do not add Cloudflare no-SLA/best-effort/dev-only UI copy; risk/limitation copy is docs-only.
  Parallelization: Wave 3 | Blocked by: 8 | Blocks: 11
  References: `ui/src/components/dashboard/service-status-panel.tsx:8-69`; `ui/src/components/dashboard/config-panel.tsx:171-230`; `ui/src/components/dashboard/config-core-sections.tsx:42-74`; `ui/src/components/dashboard/dashboard-shell.tsx:19-30` (read full before editing); `.omo/knowledges/tunnel-ui-requirements-2026-06-28.md:37-44`.
  Acceptance criteria: `pnpm --dir ui build`, `pnpm --dir ui lint`, and UI e2e/fixture tests for tunnel row/login/config fields exit 0. `rg -n "no SLA|best-effort|best effort|development only|dev only|Cloudflare.*SLA" ui/src` exits 1.
  QA scenarios: Happy: mocked running tunnel renders URL, click target, QR hover, and authenticated SYSTEMS config fields. Failure: tunnel + non-empty password + no token renders login prompt and blocks config form/API call until login. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-9.md` plus screenshot if visual tooling is available.
  Final verification: after this todo, run `pnpm --dir ui build`, `pnpm --dir ui lint`, and `rg -n "no SLA|best-effort|best effort|development only|dev only|Cloudflare.*SLA" ui/src`; evidence must show grep exits 1.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 10. Update Windows packaging to stage pinned `cloudflared.exe` under `tools/cloudflared/`
  What to do / Must NOT do: Modify packaging/support scripts to read pinned URL from `scripts/cloudflared-windows-amd64.url` and expected SHA-256 from `scripts/cloudflared-windows-amd64.sha256`, download `cloudflared-windows-amd64.exe` into `target/cloudflared-cache/cloudflared-windows-amd64.exe` when no verified cache exists, verify SHA-256 before staging, reuse verified cache offline, and fail if neither network nor valid cache is available. Stage it exactly as `ed-sentry/tools/cloudflared/cloudflared.exe` in `dist/ed-sentry` and the zip. Add committed `third_party/cloudflared/LICENSE.txt` from upstream Apache-2.0 text and stage it as `ed-sentry/tools/cloudflared/LICENSE-cloudflared.txt`. Update runtime lookup to prefer executable sibling `tools/cloudflared/cloudflared.exe` in packaged mode and support an explicit dev/env override for tests. Do not place `cloudflared.exe` in package root. Do not commit the binary unless user explicitly approves.
  Parallelization: Wave 4 | Blocked by: 5 | Blocks: 11, final package
  References: `scripts/package-windows-gnu.sh:75-109`; project `AGENTS.md` packaging rule; `.omo/knowledges/tunnel-service-options-2026-06-28.md:47-53`; Cloudflare release/download/license findings in `.omo/drafts/tunnel-cloudflare-quick.md:48-50`; packaging pin requirements in this plan at `.omo/plans/tunnel-cloudflare-quick.md:65-67`.
  Acceptance criteria: `./scripts/package-windows-gnu.sh` exits 0 and `unzip -l dist/ed-sentry-x86_64-pc-windows-gnu.zip | rg 'ed-sentry/tools/cloudflared/cloudflared\.exe$'` exits 0. Script output includes SHA-256 for `tools/cloudflared/cloudflared.exe` in addition to existing expected artifacts.
  QA scenarios: Happy: package contains exact subfolder path and runtime lookup resolves it. Failure: checksum mismatch aborts packaging before zip is published/staged. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-10.md`.
  Final verification: after this todo, run `./scripts/package-windows-gnu.sh` and `unzip -l dist/ed-sentry-x86_64-pc-windows-gnu.zip | rg 'ed-sentry/tools/cloudflared/cloudflared\.exe$'`; evidence must include SHA-256 output and license file path in zip listing.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 11. Add integration/e2e QA for tunnel HTTP, WebSocket, auth, and UI scope
  What to do / Must NOT do: Add automated tests using a fake/mock tunnel provider wherever possible; add ignored/manual real Cloudflare tunnel smoke test only if safe and clearly optional. Verify HTTP status/config endpoints and `/ws`/`/api/events` behavior through active tunnel host logic. Verify no unrelated UI source changed beyond allowed files by reviewing diff and using targeted grep/snapshot assertions. Do not require live Cloudflare in normal CI/test suite.
  Parallelization: Wave 4 | Blocked by: 6, 9, 10 | Blocks: final verification
  References: `tests/webui/server.rs` and `tests/webui/config_write.rs` (read before editing); `src/web/policy/ws.rs:38-111`; `ui/e2e/*` (inspect before editing); strict UI scope `.omo/knowledges/tunnel-ui-requirements-2026-06-28.md:37-44`.
  Acceptance criteria: `cargo test --all tunnel_websocket tunnel_auth web_policy config_api`, `pnpm --dir ui build`, and any updated `pnpm --dir ui test:e2e` or fixture command pass. Optional ignored real tunnel test is documented but not required for default pass.
  QA scenarios: Happy: fake tunnel host can load dashboard and connect WebSocket; authenticated token can save config. Failure: protected config route without token fails under tunnel host; unrelated host rejected; UI disclaimer grep has no matches. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-11.md`.
  Final verification: after this todo, run `cargo test --all tunnel_websocket`, `cargo test --all tunnel_auth`, `cargo test --all web_policy`, `cargo test --all config_api`, and `pnpm --dir ui build`; evidence must include fake tunnel WebSocket success and protected route failures.
  Commit: N | plan does not require commits unless user explicitly asks later.

- [x] 12. Update docs/config template/knowledge while preserving secret safety and UI disclaimer constraint
  What to do / Must NOT do: Update `config.example.toml`, README, release/package docs, and a final `.omo/knowledges/tunnel-cloudflare-quick-implementation-notes-<date>.md` with `[tunnel]` usage, auto_start/manual start, `config_password` empty-risk semantics, Windows bundled cloudflared subfolder, and provider scope. `.omo/knowledges` updates are required only as internal project memory, not user-facing product docs. Docs may mention Cloudflare Quick Tunnel operational limitations if needed, but React UI source must not show no-SLA/best-effort/dev-only disclaimer text. Do not include real passwords/tokens or raw private paths. Do not change project rules/AGENTS unless explicitly requested.
  Parallelization: Wave 4 | Blocked by: 1, 10 | Blocks: final verification
  References: `config.example.toml:62-83`; `README.md` WebUI/config/package sections; `.omo/knowledges/tunnel-ui-requirements-2026-06-28.md`; `.omo/knowledges/tunnel-service-options-2026-06-28.md`.
  Acceptance criteria: docs/config compile with no secrets; `rg -n --hidden --glob '!target/**' --glob '!ui/node_modules/**' --glob '!ui/dist/**' --glob '!dist/**' 'access_token\s*=\s*"[^"<][^"]{8,}"|config_password\s*=\s*"[^"<][^"]{8,}"|BEGIN (RSA|OPENSSH|PRIVATE) KEY' README.md config.example.toml src ui .omo/knowledges` exits 1 or only reports intentional safe placeholders.
  QA scenarios: Happy: `config.example.toml` has `[tunnel]` defaults with empty password placeholder and no real secret. Failure: secret scan catches any non-placeholder password/token before final. Evidence `.omo/evidence/tunnel-cloudflare-quick/task-12.md`.
  Final verification: after this todo, run the documented secret scan and `rg -n "no SLA|best-effort|best effort|development only|dev only|Cloudflare.*SLA" ui/src`; evidence must show no UI disclaimer matches and no real password/token placeholders beyond safe examples.
  Commit: N | plan does not require commits unless user explicitly asks later.

## Final verification wave
> Runs in parallel after ALL todos. ALL must APPROVE. Surface results and wait for the user's explicit okay before declaring complete.
- [x] F1. Plan compliance audit
  - Verify every Must Have and Must NOT Have above is satisfied against the final diff.
  - Commands/evidence: record checklist in `.omo/evidence/tunnel-cloudflare-quick/final-plan-compliance.md`; include exact `git diff --stat` and allowed UI files list.
  - Must approve only if UI edits are limited to approved surfaces and SSH provider is inert.
- [x] F2. Code quality review
  - Run:
    ```bash
    cargo fmt --check
    cargo test --all
    cargo clippy --all-targets --all-features -- -D warnings
    pnpm --dir ui build
    pnpm --dir ui lint
    ```
  - Evidence: `.omo/evidence/tunnel-cloudflare-quick/final-code-quality.md` with command outputs and pass signals.
  - Must approve only if no warnings/errors remain and no secret appears in logs.
- [x] F3. Real functional QA
  - Run package and tunnel-oriented checks:
    ```bash
    ./scripts/package-windows-gnu.sh
    unzip -l dist/ed-sentry-x86_64-pc-windows-gnu.zip | rg 'ed-sentry/tools/cloudflared/cloudflared\.exe$'
    rg -n "no SLA|best-effort|best effort|development only|dev only|Cloudflare.*SLA" ui/src
    ```
  - Expected: package command regenerates zip and prints SHA-256 lines for `ed-sentry.exe`, `ed-sentry-core.exe`, `WebView2Loader.dll`, `webui/index.html`, and `tools/cloudflared/cloudflared.exe`; unzip path check passes; UI disclaimer grep exits 1 with no matches.
  - Evidence: `.omo/evidence/tunnel-cloudflare-quick/final-functional-qa.md`.
- [x] F4. Security/scope fidelity
  - Verify auth matrix with targeted tests and diff inspection: empty password bypasses tunnel auth; non-empty password protects SYSTEMS/config APIs under tunnel host; valid JWT permits config mutation; invalid/expired token rejects; local/Tauri behavior unchanged; unknown Host rejected.
  - Run secret/privacy scan command from README adapted to include `[tunnel]` password patterns.
  - Evidence: `.omo/evidence/tunnel-cloudflare-quick/final-security-scope.md`.

## Commit strategy
- Do not commit unless the user explicitly asks. If asked later, use small atomic commits after full verification:
  1. `feat(config): add tunnel configuration model`
  2. `feat(tunnel): add cloudflare quick tunnel runtime`
  3. `feat(web): add tunnel auth and APIs`
  4. `feat(ui): add tunnel controls and gated systems access`
  5. `build(package): bundle cloudflared in windows artifact`
  6. `docs(tunnel): document quick tunnel configuration`
- Before any commit, run `git status`, `git diff`, `git log --oneline -10`, inspect staged files only, and run a secret scan. Never commit generated `dist/` artifacts or real credentials unless repository policy explicitly tracks them.

## Success criteria
- `[tunnel]` config exists, defaults correctly, writes correctly, and does not echo non-empty `config_password` through config APIs.
- WebUI enabled implies tunnel can be used; auto_start starts automatically and manual START works from Service Nodes.
- Running tunnel exposes a public `https://*.trycloudflare.com` URL, visible in TUNNEL status, clickable, and QR-rendered on hover.
- Cloudflare provider is the only startable Phase 1 provider; SSH is future/inert only.
- Tunnel auth behavior exactly matches password policy: empty password open; non-empty password requires JWT-like login for SYSTEMS/config page and APIs; authenticated tunnel users can modify config.
- `/ws`/`/api/events` WebSocket live updates work through tunnel host logic.
- Local loopback WebUI and desktop/Tauri behavior remain unchanged.
- Unknown/unrelated Host remains rejected; active tunnel host is accepted only for the current tunnel session.
- UI changes are limited to SYSTEMS login prompt, Service Nodes TUNNEL entry/link/QR, and SYSTEMS config tunnel fields.
- React UI contains no Cloudflare no-SLA/dev-only/best-effort disclaimer text.
- Windows GNU package contains `ed-sentry/tools/cloudflared/cloudflared.exe` and hash output includes it.
- Full verification commands pass, including `./scripts/package-windows-gnu.sh`.
