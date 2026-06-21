# Current project progress - 2026-06-20

- Repository state: branch `feature/ui` points at `40500c4` and matches `origin/master`/`master`; no upstream is configured for `feature/ui`.
- Working tree state during review: no tracked file diffs; untracked `.codegraph` exists from the background CodeGraph bootstrap.
- Product scope: `ed-sentry` is a Rust CLI for Elite Dangerous Journal AFK monitoring. Current README scope is local CLI monitoring plus optional watch-mode Matrix delivery. Replay remains terminal-only.
- Implemented runtime paths: Journal discovery, watch preload and live tail, replay streaming, event parsing, session/mission state, warning scheduling, terminal rendering, dynamic title support, delivery fanout, and Matrix SDK notification/status delivery for watch mode.
- Implemented configuration: `config.example.toml` is the safe committed template; local root `config.toml` is gitignored and can contain Matrix credentials. Config precedence is CLI flags, explicit or implicit config file, then defaults.
- Matrix constraints: delivery is optional, best-effort, watch-mode only, and uses unencrypted rooms. E2EE, Matrix command handling, Discord, WebUI, EDMC, durable queues, automation/relog, and dashboards remain out of scope.
- CI/release: GitHub Actions CI runs fmt, clippy, and tests on Ubuntu and Windows. Tag releases build `ed-sentry-x86_64-unknown-linux-gnu.tar.gz` and `ed-sentry-x86_64-pc-windows-msvc.zip`.
- Verification run on 2026-06-20: `cargo fmt --check`, `cargo test --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line`, and `git check-ignore config.toml` all exited 0.
- `cargo test --all` passed 171 non-ignored tests; `tests/real_journal_replay.rs` contains one ignored local-only real Journal test.
- Manual CLI replay output showed startup, `Scan`, `Kill`, cargo loss, fuel, `Total Stats`, and `Monitor stopped` fragments from `tests/fixtures/journal_combat_bounty.log`.
- Known residual notes from prior task records: `.gitignore` still lacks some broad sensitive-file patterns such as keystores, `.netrc`, `.pgpass`, `*.secret`, `.boto`, `.s3cfg`, and `kubeconfig`; this is not blocking current tests but is relevant before future credential-heavy work.
