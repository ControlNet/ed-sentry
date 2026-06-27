# Task 16 CI and Release Workflows

- CI lives in `.github/workflows/ci.yml` and runs on both `push` and `pull_request` for `ubuntu-latest` and `windows-latest`.
- CI installs stable Rust with `rustfmt` and `clippy`, caches Cargo registry/git data plus `target/`, then runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all`.
- Release artifacts live in `.github/workflows/release.yml` and trigger only on tags matching `v*`.
- Release builds are native per runner: Linux uses `ubuntu-latest`; Windows uses `windows-latest`. Phase 1 intentionally does not cross-compile Windows from Linux.
- Locked artifact names are `ed-sentry-x86_64-unknown-linux-gnu.tar.gz` and `ed-sentry-x86_64-pc-windows-msvc.zip`.
- Workflows must not invoke `real_journal_replay -- --ignored` and must not reference personal Journal folders; those remain local-only verification paths.
