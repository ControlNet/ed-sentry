# Version Banner Build Info

For CLI, terminal title, and Matrix startup version display, Rust now uses
`src/build_info.rs` as the shared source of truth:

- `APP_VERSION` is `CARGO_PKG_VERSION`.
- `APP_COMMIT_DATE` is `ED_SENTRY_COMMIT_DATE` from `build.rs`.
- `APP_BUILD_VERSION` is `APP_VERSION-APP_COMMIT_DATE`.
- `app_title()` renders `ed-sentry {APP_BUILD_VERSION}`.

`build.rs` queries the latest git commit date with:

```bash
git log -1 --format=%cd --date=format:%Y%m%d
```

This mirrors the GUI build version format in `ui/vite.config.ts`.

Verification commands used for the banner alignment change:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
./scripts/package-windows-gnu.sh
```

Manual QA used `target/debug/ed-sentry-core` with replay/watch surfaces and
`ED_AFK_DASHBOARD_FAKE_MATRIX_LOG` to confirm the Matrix startup remote text
contains `Version: {APP_BUILD_VERSION}`.
