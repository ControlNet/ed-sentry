## 2026-06-28 Task: start-work
Plan approved after high-accuracy review. Phase 1 scope is Cloudflare Quick Tunnel only, with inert future SSH shape. Product code edits must be delegated; orchestrator only manages .omo state, plan checkboxes, and verification gates.

## 2026-06-28 Task: 4 tunnel auth primitives
The available registry exposes `jsonwebtoken` 10.4.0 rather than v11; use `default-features = false` plus `rust_crypto` explicitly. The primitive stays data-only and uses an in-memory per-run `getrandom` HMAC secret, so route/frontend integration can later depend on `Authorization: Bearer <token>` without persisting signing material.

## 2026-06-28 Task: 3 independent verification
Task 3 is verified as config DTO/write/frontend schema work only. Current branch diff also contains concurrent task-4/auth/provider changes in manifests and app tunnel exports; do not credit those broader route/auth/provider primitives to task 3 when updating the plan.

## 2026-06-28 Task: 4 independent verification
Task 4 is independently confirmed as primitive-only tunnel auth work: `jsonwebtoken` 10.4.0 is configured with explicit `rust_crypto`, `TunnelAuth::new_per_run()` keeps HMAC signing material in memory, tokens carry subject/purpose/iat/exp/active host/session claims, focused `cargo test --all tunnel_auth` passes, and no route/UI/Tauri/Cloudflare process integration should be credited to this task.

## 2026-06-28 Task: 5 Cloudflare provider/lifecycle
Process tests should use deterministic fake `cloudflared` scripts and never invoke Cloudflare. The provider command shape is locked by tests as `tunnel --url http://127.0.0.1:<port>`, with URL parsing from both stdout and stderr. For crash tests, make the fake process exit immediately after emitting the URL; fractional `sleep` timing is not deterministic enough for refresh assertions. Avoid parallel Cargo verification for this repo when collecting evidence because lock contention can make otherwise passing commands time out.

## 2026-06-28 Task: 5 Atlas idempotency fix
Provider `start_for_port()` refreshes the child before idempotency checks by design, because a crashed child must clear stale active host/session before restart. Therefore idempotency tests must keep the fake process alive deterministically after URL emission; prefer safely quoted log paths plus `while :; do sleep 1; done` over one-shot or fragile shell snippets such as `printf '%s\n' started >> ...; sleep 30`.

## 2026-06-28 Task: 7 Atlas stderr provider fix
Provider success-path fake processes that prove stdout/stderr URL parsing must remain alive deterministically after emitting the URL and need enough timeout budget for loaded full-suite scheduling. `sleep 30` and 150 ms fake-provider timeouts can still race the output-reader delivery under `cargo test --all tunnel`; use `while :; do sleep 1; done` for running-success fakes and keep timeout budgets realistic while preserving `Running` assertions.

## 2026-06-28 Task: 8 frontend adapter auth
Web tunnel auth state is intentionally a single `sessionStorage` record under `ed-sentry:tunnel-auth-token` containing the bearer value plus active public host/session binding. Config GET/PUT rechecks unauthenticated tunnel status before adding the header so stale sessions are cleared before protected config requests. Secret Guard currently flags the existing Rust redaction fixture `src/app/config.rs:151` (`fixture-tunnel-password`) as a password-shaped test value; treat it as a reviewed fixture false positive unless that line changes.

## 2026-06-28 Task: 9 tunnel UI surfaces
The active Service Nodes dashboard surface is `ui/src/components/dashboard/tactical-telemetry-view.tsx` plus `tactical-telemetry-widgets.tsx`; `service-status-panel.tsx` is an older unused component with no current callers. Tunnel UI tests should target the tactical Service Nodes region. No QR dependency existed before task 9, so local QR rendering uses the added `qrcode` package instead of an external QR service.

## 2026-06-28 F3 final functional QA
The underscore-named package path `dist/ed-sentry_x86_64-pc-windows-gnu.zip` was absent during F3, so the plan hyphen path `dist/ed-sentry-x86_64-pc-windows-gnu.zip` was used. The rebuilt zip contained `ed-sentry/tools/cloudflared/cloudflared.exe` and `ed-sentry/tools/cloudflared/LICENSE-cloudflared.txt`, and the package script printed the required cloudflared SHA-256 line from the verified cache.

## 2026-06-28 Final plan compliance audit
F1 rejected because configured `provider = "ssh"` is only handled by startup policy; manual start via Web API/desktop goes through `WebTunnelState::start()` -> `TunnelLifecycleManager::manual_start()` -> Cloudflare `start_for_port()` without checking provider. Add regression coverage that `provider = "ssh"` manual start returns `unsupported` and does not invoke the fake cloudflared executable.

## 2026-06-28 F1 SSH manual-start fix
Fix stores the configured tunnel provider in `TunnelLifecycleManager` and initializes `WebTunnelState` from `RuntimeConfig`, so startup policy, Web API manual start, and desktop manual start all return `unsupported` for `provider = "ssh"`. Regression coverage exists at lifecycle, Web route, and public desktop command boundaries (`tests/tunnel_desktop.rs`); lifecycle fake URL timeout is 2 seconds to avoid the prior 150 ms output-reader race.

## 2026-06-28 F2 tunnel provider early-exit fix
`spawn_cloudflared()` can time out under loaded full-suite scheduling after an immediately exited fake process, so the timeout branch must call `child.try_wait()` before reporting a generic no-URL timeout. This preserves the deterministic retryable message `cloudflared exited before URL was reported: <status>` for early exits without weakening the no-URL timeout path.

## 2026-06-28 F1 rerun approval
F1 rerun after the SSH manual-start fix is approved: `TunnelLifecycleManager` stores `configured_provider`, `WebTunnelState::for_config()` seeds it from `RuntimeConfig`, and lifecycle/Web route/desktop tests prove `provider = "ssh"` manual start returns unsupported without invoking fake cloudflared. UI scope remains limited to tunnel surfaces/plumbing and the no-disclaimer scan exits 1.
