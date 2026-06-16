# Package rename to ed-sentry

- Package and binary name are now `ed-sentry`; the current checkout directory may still be named from the old repository path until the user/remote host renames it outside Cargo.
- Rust crate imports use `ed_sentry` because Cargo maps hyphens to underscores.
- Functional source/docs/workflows should have no previous package-name or crate-name references. Current `.omo` knowledge, plan, and notepad paths/content were also renamed to avoid future agent confusion.
- Ignored local release artifacts were renamed from previous artifact prefix to `ed-sentry*` for directory-level consistency.
