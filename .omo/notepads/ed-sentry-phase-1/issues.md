## 2026-06-09T14:17:20+00:00 - Task 2/4/6 parallel verification clippy failure
- `cargo clippy --all-targets --all-features -- -D warnings` failed because the binary privately declared modules with public helper APIs not used by `main.rs`, causing dead-code errors for downstream-task APIs in `state` and `time`.
- Fixed by adding `src/lib.rs` with public module exports and importing Task 2 config types from `ed_sentry::config` in the binary instead of private `mod` declarations.
- Replaced the manual delegating `Default` impl for `AppConfig` with `#[derive(Default)]`; nested config defaults remain custom, preserving the locked config values.

## 2026-06-09T00:00:00+00:00 - F4: Scope fidelity privacy hardening note
- Secret-guard tracked-file scan was clean and F4 found sanitized fixtures plus ignored/read-only optional real Journal replay boundaries, so this is not a scope blocker. However, the gitignore coverage audit reports common sensitive patterns are not ignored (, , , , , etc.); consider adding them before future Matrix/credential work, and consider local real-Journal ignore patterns if real samples may be placed inside the repo.
- Correction: the omitted sensitive patterns reported by secret-guard include .env, *.pem, *.key, *.p12, *.pfx, credentials.json, token.json, id_rsa, id_ed25519, secrets.yml, and secrets.yaml.
