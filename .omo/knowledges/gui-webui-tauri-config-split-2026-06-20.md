# GUI WebUI Tauri Config Split

On 2026-06-20, `src/config.rs` was split into responsibility-named modules under `src/config/` while preserving the public `ed_sentry::config::*` API through re-exports.

Responsibility map:
- `config.rs`: module declarations and public re-exports.
- `model.rs`: aggregate config structs.
- `journal.rs`, `monitor.rs`, `log_levels.rs`, `matrix.rs`, `web.rs`: section-specific types/defaults/parsing.
- `read.rs`: config loading and TOML section dispatch.
- `runtime.rs`: CLI override application.
- `error.rs`: config load/parse errors.
- `value_read.rs`: typed TOML primitive readers.
- `source.rs`: config source/path/write-state model.
- `write.rs` and `write/*`: editable config persistence.

Verification evidence is under `.omo/evidence/gui-webui-tauri/fix-config-split-*`.
