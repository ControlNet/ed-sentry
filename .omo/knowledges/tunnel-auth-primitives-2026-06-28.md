# Tunnel auth primitive notes - 2026-06-28

- Task 4 adds only data-level auth primitives; route enforcement, protected Web API policy, frontend token storage, and Tauri wiring remain later tasks.
- `jsonwebtoken` is configured as `version = "10.4.0", default-features = false, features = ["rust_crypto"]` because the local registry does not expose v11 yet.
- `TunnelAuth::new_per_run()` generates an in-memory 32-byte HMAC secret via `getrandom`; the primitive has no disk persistence path.
- Tokens are accepted only through `Authorization: Bearer <token>` by `validate_authorization` when `config_password` is non-empty.
- Empty `config_password` returns `TunnelAuthValidationResult::Bypassed`, matching the accepted-risk requirement.
- Token claims bind subject, purpose, iat, exp, active tunnel host, and active tunnel session id; stale host/session and app restarts reject old tokens.
