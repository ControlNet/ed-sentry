# Task 2 CLI and Config Contract

- `src/main.rs` owns the clap parser and runtime-mode validation. There are no `watch`/`replay` subcommands; mode is selected by the top-level `--replay` flag.
- If `--replay` is absent, `build_runtime_command` runs watch mode directly. Replay requires `--set-file`, rejects `--journal`, and rejects `--poll-interval-ms` to avoid a no-effect option.
- `src/config.rs` loads TOML as `toml::Value`, applies known keys manually, warns on wrong-typed keys while keeping defaults, and returns a malformed-TOML error that the app maps to exit code 1.
- Effective precedence is CLI overrides, then `--config` TOML, then locked defaults. `--no-status-line` maps to `monitor.live_status = false`; top-level `--poll-interval-ms` overrides TOML/default `monitor.poll_interval_ms` in watch mode.
