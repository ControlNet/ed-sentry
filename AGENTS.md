## Project Rules

- Code, comments, and identifiers are English. Reply to the user in their language unless they ask otherwise.
- Prefer minimal, reviewable patches and do not revert user changes.
- For non-trivial changes, include exact verification commands and expected pass signals.
- Never use `git clean`.
- Never include secrets, tokens, or keys in code, logs, docs, or examples.
- Do not modify `reference-design/design1.tsx`; it is a read-only design reference.
- When frontend, Tauri, packaging, or WebUI asset files change, rebuild the Windows artifact before reporting completion:

```bash
./scripts/package-windows-gnu.sh
```

Expected signal: `dist/ed-sentry-x86_64-pc-windows-gnu.zip` is regenerated, and the script prints SHA-256 lines for `ed-sentry.exe`, `ed-sentry-core.exe`, `WebView2Loader.dll`, and `webui/index.html`.

## Version Release Updates

- The root `Cargo.toml` package version is the only manual release version source. For a release such as `v0.1.1`, first update only `Cargo.toml`, then sync Tauri metadata with:

```bash
node scripts/sync-release-version.mjs --check-tag v0.1.1
```

Expected signal: the command prints `Synced release version 0.1.1` and updates `ui/src-tauri/Cargo.toml` plus `ui/src-tauri/tauri.conf.json`.

- After a version bump, refresh lockfiles and any version-pinned docs/tests. In particular, keep `Cargo.lock`, `ui/src-tauri/Cargo.lock`, `scripts/release-package-contract.test.mjs`, and README release artifact examples consistent with the new version.
- Before committing a version bump, run:

```bash
node --test scripts/*.test.mjs
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
./scripts/package-windows-gnu.sh
```

Expected signals: Node tests pass, formatting is clean, clippy emits no warnings, Rust tests pass, and the Windows packaging script regenerates `dist/ed-sentry-x86_64-pc-windows-gnu.zip` with SHA-256 lines for `ed-sentry.exe`, `ed-sentry-core.exe`, `WebView2Loader.dll`, and `webui/index.html`.

- Do not create or push the release tag until the version bump commit is on `master` and the latest `master` CI run has completed successfully.
- Release tags must match the root Cargo version exactly, for example `v0.1.1` for `version = "0.1.1"`. The release workflow rejects mismatched tags through `scripts/sync-release-version.mjs --check-tag`.
- When staging a release bump, include only the intended version-related files and do not stage unrelated untracked files such as `reference-design/design_about.tsx`.
