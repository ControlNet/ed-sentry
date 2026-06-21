# GUI WebUI Task 2 Post-Refactor Gate

- 2026-06-20 post-refactor re-gate found two clippy blockers in `src/config.rs`: `RuntimeConfig` manual `Default` was derivable, and inline `toml::de::Error` in `ConfigError::MalformedToml` triggered `clippy::result_large_err` across config-loading functions.
- Behavior-preserving fix: derive `Default` on `RuntimeConfig`, remove the manual impl, change `MalformedToml.source` to `Box<toml::de::Error>`, and wrap parse errors with `Box::new(parse_error)`.
- Verification passed after the fix: `cargo fmt --check`, focused Web defaults/source/no-server/wrong-typed tests, `cargo test --all`, and `cargo clippy --all-targets --all-features -- -D warnings`.
- Todo 2 post-fix evidence reports are `.omo/evidence/gui-webui-tauri/task-2-code-review.md` and `.omo/evidence/gui-webui-tauri/task-2-manual-qa-matrix.md`.
- Residual risk remains: legacy `src/config.rs` is still large at 1265 pure LOC, while extracted `src/config/web.rs` and `src/config/source.rs` are under 250 pure LOC.
