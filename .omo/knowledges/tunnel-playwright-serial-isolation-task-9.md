# Tunnel Playwright Serial Isolation - Task 9

Task 9's `ui/e2e/tunnel-ui.spec.ts` drives multiple mock tunnel scenarios in one file, including repeated same-page navigations between `mock_state=tunnel_start`, `mock_state=tunnel_disabled`, and `mock_state=tunnel_running`.

If CI or a remote verifier reports `net::ERR_CONNECTION_REFUSED` during an in-spec navigation while the local spec does not reproduce, prefer an e2e-only isolation fix before changing product code. In this case, adding `test.describe.configure({ mode: "serial" })` at the top of the spec preserved all START/RUNNING/UNAVAILABLE/QR/auth assertions and made the normal run use one worker for the file.

Evidence was appended to `.omo/evidence/tunnel-cloudflare-quick/task-9.md` under `Retry Evidence - Tunnel UI Playwright Flake`.
