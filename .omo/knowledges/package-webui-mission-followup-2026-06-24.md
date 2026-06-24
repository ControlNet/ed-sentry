# Package/WebUI/Mission follow-up notes - 2026-06-24

- Root Cargo package can be renamed to `ed-sentry-core` while preserving the public Rust library crate name with `[lib] name = "ed_sentry"`; this avoids rewriting all `ed_sentry::...` test and production imports.
- The Tauri launcher package can own the user-facing package/binary name `ed-sentry`; the launcher must spawn sibling `ed-sentry-core(.exe) --gui`.
- Windows GNU package layout after the rename is `ed-sentry/ed-sentry.exe` for the GUI launcher plus `ed-sentry/ed-sentry-core.exe` for the backend/CLI binary.
- Remote WebUI config writes are controlled in two places: `WebEndpointPolicy::new()` / `authorize_state_change()` for runtime API policy, and `src/config/write/apply.rs::validate_update()` for persisted `web.host` updates.
- After allowing remote WebUI config writes, also remove stale user-facing warning text from `src/config/web.rs`, `src/web/server.rs`, and `ui/src/components/dashboard/config-core-sections.tsx`; otherwise CLI/GUI can still claim writes are disabled even when API policy allows them.
- Mission completion should be retained in `MissionTracker` for reward/history semantics, but `MissionListView::from_tracker()` should only expose `Active` and `Redirected` missions in active `items`.
