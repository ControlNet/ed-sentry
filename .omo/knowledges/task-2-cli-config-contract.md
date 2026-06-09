# Task 2 CLI and Config Contract

- `src/main.rs` owns the clap parser and runtime-mode validation. Global flags are declared with `global = true`, so they are accepted before or after `watch`/`replay`.
- No subcommand is treated as `watch` by `build_runtime_command`; the current runtime path only prints loaded configuration stubs and does not start journal tailing.
- `src/config.rs` loads TOML as `toml::Value`, applies known keys manually, warns on wrong-typed keys while keeping defaults, and returns a malformed-TOML error that the app maps to exit code 1.
- Effective precedence is CLI overrides, then `--config` TOML, then locked defaults. `--no-status-line` maps to `monitor.live_status = false`; `watch --poll-interval-ms` overrides TOML/default `monitor.poll_interval_ms`.
