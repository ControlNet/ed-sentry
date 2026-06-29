# ed-sentry

`ed-sentry` is an independent Rust CLI for Elite Dangerous Journal AFK monitoring use cases. It reads local Journal files, tracks Phase 1 combat and session signals, and prints terminal output for live watch and replay runs.

This project is not a fork, port, or copy of another monitor. The implementation, code structure, messages, and docs are written for this repository while following Elite Dangerous Journal semantics and the Phase 1 plan.

## Phase 1 Scope

Phase 1 is local CLI monitoring with Matrix watch-mode delivery.

Supported now:

- Find the newest `Journal.*.log` file from a configured folder.
- Watch a selected Journal file, print matching events already present in that file, then tail appended complete lines.
- Replay one sanitized or local Journal file from start to end.
- Track cargo scans, observed kills, bounties, massacre mission progress, shield and hull state, fighter events, fuel reports, cargo loss, death, and session summaries.
- Update the AFK `Checklist` from the selected Journal file plus local `Status.json` and `Cargo.json` companion files during watch-capable runs.
- Render terminal event logs and a live status line when the output is a TTY.
- Send watch-mode notifications to an unencrypted Matrix room when `[matrix] enabled = true` is configured.
- Start the local WebUI dashboard in watch-capable runtimes when `[web] enabled = true` is configured.
- Build the shared Web/Tauri frontend under `ui/`, including the local `ed-sentry` desktop launcher entry.

Out of scope for Phase 1:

- Matrix command handling.
- Discord delivery.
- Replay inside the WebUI or desktop dashboard.
- EDMC plugin support.
- auto relog, key simulation, game automation, and relog scripting.
- Database storage, multi-day stored analytics, or chart-library dashboards.

## Journal Paths

On Windows, the default Journal folder is resolved from the system Saved Games known folder, so it follows Windows folder relocation settings:

```text
<Saved Games>\Frontier Developments\Elite Dangerous
```

On Linux and development machines, pass the folder or a file explicitly:

```bash
cargo run -- --journal "/path/to/Elite Dangerous"
cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
```

By default, `ed-sentry` runs in watch mode and accepts `--journal <folder>` or `--set-file <file>`. Passing `--replay` switches to replay mode; replay requires `--set-file <file>` and rejects `--journal` in Phase 1.

## CLI Usage

Run from the repository root while developing:

```bash
cargo run -- --journal "/path/to/Elite Dangerous"
```

Run watch mode with an explicit config file:

```bash
cargo run -- --config config.toml --journal "/path/to/Elite Dangerous"
```

Replay the deterministic sanitized combat fixture without Matrix:

```bash
cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line
```

Run the normal test suite:

```bash
cargo test --all
```

Build the shared WebUI/Tauri frontend:

```bash
pnpm --dir ui build
```

Build the desktop GUI locally when the host has Tauri v2 system dependencies installed:

```bash
pnpm --dir ui tauri:build
pnpm --dir ui tauri build
```

Run the full local verification set before sending changes for review:

```bash
cargo fmt --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
```

Run the optional ignored real Journal regression test:

```bash
cargo test --test real_journal_replay -- --ignored
```

Expected signals:

- The replay fixture exits `0` and prints reference-style terminal fragments such as `Scan`, `Kill`, and `Total Stats`.
- `cargo test --all` exits `0` without requiring private Journal files.
- `pnpm --dir ui build` exits `0` and writes `ui/dist`.
- `cargo fmt --check` exits `0` when formatting is current.
- `cargo clippy --all-targets --all-features -- -D warnings` exits `0` when no lint warnings remain.
- The ignored real Journal test exits `0` when local Journals exist. If that test is unavailable in the current checkout, Task 14 hasn't added it yet.

Common flags:

- `--config <file>` loads a TOML config file.
- `--journal <folder>` sets the Journal folder for `watch`.
- `--set-file <file>` selects one Journal file.
- `--file-select` lists recent Journal files and reads the selected number from stdin.
- `--reset-session` clears watch counters after startup preload output. In replay it is accepted for compatibility and prints one no-effect warning.
- `--debug` prints runtime diagnostics such as selected Journal file and preload offsets.
- `--no-status-line` disables the live status line and keeps output newline-safe.
- `--poll-interval-ms <ms>` changes the fallback and housekeeping cadence in the default watch mode. Watcher events drive low-latency updates when available. `--replay` rejects this flag because it has no replay effect.
- `--replay` reads the selected Journal file from start to finish and exits.

No subcommands are used. If `--replay` is absent, the CLI runs in watch mode.

In watch-capable terminal, WebUI, and desktop runtimes, file watcher events from the selected Journal file, `Status.json`, and `Cargo.json` update the current snapshot without waiting for the interval. The watcher filters to those files in the selected Journal folder; unrelated Journal files are ignored. `poll_interval_ms` remains the fallback and housekeeping cadence for missed watcher events, status publication, and warning checks.

The WebUI and desktop dashboard show a `Checklist` panel with tri-state `PASS`/`FAIL`/`UNKNOWN` values for `Hardpoints deployed`, `Engine pips zero`, and `Cargo loaded`. `Hardpoints deployed` and `Engine pips zero` come from `Status.json`. `Cargo loaded` means non-empty ship cargo from `Cargo.json`, not cargo market value.

## Configuration

Use `config.example.toml` as the committed reference. Copy it to `config.toml` for local settings:

```bash
cp config.example.toml config.toml
```

Only these config names are supported by this project documentation:

- `config.example.toml` is committed and safe to share.
- `config.toml` is local, gitignored, and must not be committed because it can contain your Matrix access token.

Config precedence is:

1. CLI flags.
2. Values from an explicit `--config <file>`, or from `./config.toml` when `--config` is not passed and that file exists.
3. Built-in defaults.

If `--config <file>` is passed, that path is strict: the file must exist and be valid TOML. If `--config` is not passed, the app auto-loads `./config.toml` when present. If `./config.toml` is absent, the app runs with built-in defaults.

Missing keys keep their defaults. Wrong typed keys print a warning and keep the default for that key. Malformed TOML exits with code `1`.

Important monitor defaults:

- `live_status = true` enables the TTY status line unless `--no-status-line` is passed.
- `use_utc = false` prints local time by default.
- `poll_interval_ms = 1000` is the default fallback and housekeeping interval for watch mode.
- `warn_kill_rate = 20`, `warn_no_kills_initial_minutes = 5`, `warn_no_kills_minutes = 20`, and `warn_cooldown_minutes = 30` control idle and low-rate warnings.
- `duplicate_max = 5` is retained for future remote delivery controls; Phase 1 terminal output does not suppress duplicate notifications so it stays aligned with the upstream console stream.
- `pirate_names = false`, `bounty_faction = false`, and `bounty_value = false` keep default cargo-scan and kill lines concise; set them to `true` to include pilot names, victim factions, and credit values.
- `extended_stats = false` keeps default event lines concise; set it to `true` to include supported event counters such as kill sequence numbers.

WebUI settings live in `[web]`:

- `enabled = false` keeps WebUI disabled by default. Set `enabled = true` in config to start WebUI from watch-capable CLI and desktop runtimes.
- `host = "127.0.0.1"` binds to loopback by default for local-only access.
- `port = 8765` is the default WebUI port.
- `open_browser = false` avoids launching a browser automatically.
- Non-localhost `host` values print a warning and continue so deliberate advanced binds are visible but do not block startup. Treat any non-loopback bind as an advanced local-network exposure and do not use it on untrusted networks.

There is no separate CLI switch for WebUI startup. Configuration is the startup contract.

Replay remains terminal-only and ignores WebUI by design. Even when `[web] enabled = true`, replay does not initialize WebUI, start a server, open a browser, or publish WebUI status.

The first milestone uses a local-first WebUI security model:

- Static dashboard assets and read-only status endpoints may be served from the configured bind address.
- Config mutation is allowed for trusted WebUI clients on the configured bind address, including deliberate non-loopback binds.
- Config update requests are also checked against host/origin policy. Do not expose the WebUI publicly unless a later authenticated remote mode is designed.
- The config editor uses the same sanitized config view as the backend APIs. Matrix token values are never echoed back to the frontend; saving can keep, replace, or explicitly clear the local token without displaying the existing value.

WebUI assets are not embedded in the Rust binary in this milestone. `ed-sentry` looks for built frontend files in this order:

1. `ED_SENTRY_WEBUI_DIST`, for tests and development overrides.
2. A `webui/` directory beside the running `ed-sentry` executable, used by release archives.
3. The repo-local `ui/dist` directory, used during development after `pnpm --dir ui build`.

The browser WebUI and desktop GUI share the same React/Vite frontend. The browser path talks to the local WebUI backend; the desktop path is the `ed-sentry` Tauri launcher under `ui/src-tauri/` and uses desktop adapter code while reusing the same dashboard pages and Rust application services.

Replay summary log levels control individual summary fragments:

- `summary_kills` controls the `Kills` fragment.
- `summary_scans` controls the `Scans` fragment.
- `summary_bounties` controls the `Bounties` fragment.
- `summary_faction` controls per-victim-faction kill totals.
- `summary_merits` controls the Powerplay merits fragment.

Log level values control notification routing:

- `0` means off.
- `1` means notify.
- `2` and higher mean notify and add a Matrix mention when Matrix delivery is enabled.

Matrix settings live in `[matrix]`:

- `enabled = false` keeps Matrix delivery off.
- `homeserver`, `room_id`, and `access_token = "<token>"` configure Matrix delivery directly in `config.toml`. `room_id` accepts either a room ID such as `!roomid:example.org` or a legal Matrix room alias such as `#alerts:example.org`. The Matrix account identity is discovered from the access token at startup.
- `mention_user_id` is optional. Set it to the Matrix user ID that should be mentioned by level `2+` notifications.
- `status_update_interval_seconds = 60` controls how often watch mode may send status updates.

Store the token directly as `access_token = "<token>"` in local `config.toml`. There is no env-token config key. Don't commit `config.toml`.

Matrix delivery is watch-mode only. Replay remains terminal-only and never initializes Matrix, sends Matrix messages, or publishes Matrix status updates, even when `config.toml` contains Matrix settings.

Matrix end-to-end encryption is unsupported. Use an unencrypted Matrix room for delivery.

## Privacy And Fixtures

Raw local Journals are read-only inputs and must not be committed. This includes files under any personal Journal folder.

The committed files under `tests/fixtures/` are synthetic and sanitized. They use fake commander, system, faction, ship, mission, and message values. Keep raw commander names, carrier names, chat text, local paths, tokens, credentials, and private log content out of fixtures, docs, and evidence files.

See `tests/fixtures/README.md` for the fixture policy.

Useful local privacy scans:

```bash
privacy_pattern='access_token\s*=\s*"[^"<][^"]{8,}"|Matrix access token'
privacy_pattern="${privacy_pattern}:|Journal\.[0-9].*\.log|BEGIN (RSA|OPENSSH|PRIVATE) KEY"
rg -n --hidden --glob '!target/**' --glob '!ui/node_modules/**' --glob '!ui/dist/**' --glob '!dist/**' --glob '!Cargo.lock' --glob '!src/**/tests.rs' --glob '!src/**/tests/**' "$privacy_pattern" README.md config.example.toml src ui .omo/evidence/gui-webui-tauri
python /home/ubuntu/.codex/skills/secret-guard/scripts/scan_secrets.py tracked
python /home/ubuntu/.codex/skills/secret-guard/scripts/scan_secrets.py gitignore
```

The first `rg` command should exit `1` with no matches. It intentionally scans user-facing docs, production source, UI source, and evidence for raw Journal filenames and private-key/token patterns rather than the public game title, which appears in normal docs and fixture names. Synthetic parser and WebUI tests contain deliberate fake Journal filenames and fake token markers, so review them separately when changing fixtures.

## Release Artifacts

The tag release workflow publishes these Phase 1 CLI/WebUI artifact names:

- `ed-sentry-x86_64-unknown-linux-gnu.tar.gz`
- `ed-sentry-x86_64-pc-windows-msvc.zip`

Each archive expands to an `ed-sentry` folder containing:

- `ed-sentry` on Linux, or `ed-sentry.exe` on Windows.
- `config.toml`, copied from `config.example.toml`.
- `webui/`, copied from the current `ui/dist` build so the packaged binary can serve `/` without a repo checkout.

The packaged `config.toml` is copied from the committed safe template and must be edited locally before enabling Matrix delivery or WebUI. It must not contain a real access token in git.

The release workflow installs Node with pnpm `10.30.1`, runs `pnpm --dir ui install --frozen-lockfile`, builds `ui/dist`, copies it to `ed-sentry/webui/`, and checks that `webui/index.html` exists before uploading archives. This matches the runtime asset lookup order: `ED_SENTRY_WEBUI_DIST`, sibling `webui/`, then repo-local `ui/dist`.

For a local Windows GNU package, run:

```bash
scripts/package-windows-gnu.sh
```

This rebuilds `target/x86_64-pc-windows-gnu/release/ed-sentry-core.exe`, builds `ui/src-tauri/target/x86_64-pc-windows-gnu/release/ed-sentry.exe`, builds `ui/dist`, refreshes `dist/ed-sentry/`, copies `ui/dist` to `dist/ed-sentry/webui/`, and writes `dist/ed-sentry-x86_64-pc-windows-gnu.zip` using `config.example.toml` as the packaged `config.toml`.

Desktop GUI artifacts are not published by CI in this first milestone. Build `ed-sentry` locally with:

```bash
pnpm --dir ui tauri:build
pnpm --dir ui tauri build
```

The tracked release blocker is desktop runner coverage and platform packaging for Tauri artifacts; CLI/WebUI archives remain the CI-published release artifacts.

CI runs the normal sanitized test suite on Linux and Windows. Optional ignored real Journal regression tests remain local-only and are not part of CI or release workflows.

## Matrix Roadmap

The Matrix MVP is notification delivery only. It does not implement Matrix command handling.

Future work may expand delivery behavior after the local CLI and watch-mode Matrix path stay stable.
