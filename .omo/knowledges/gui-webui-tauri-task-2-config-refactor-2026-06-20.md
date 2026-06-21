# GUI WebUI Task 2 Config Refactor

- `src/config.rs` remains the public `crate::config` API surface and re-exports `WebConfig`, `ConfigSource`, `ConfigPath`, `ConfigWriteState`, and `ConfigBlockReason`.
- Todo 2 WebUI config logic now lives in `src/config/web.rs`: `WebConfig`, defaults, `[web]` parsing, localhost bind warning, and module-local WebConfig tests.
- Todo 2 config source/write-target metadata now lives in `src/config/source.rs`: source/path/write-state/block-reason types, source path helpers, blocked-source helpers, and module-local source tests.
- Compatibility unit tests remain at `config::tests::config_web_defaults_to_disabled_localhost` and `config::tests::config_source_tracks_write_target` so existing gate filters still run exactly one test.
- LOC evidence for this refactor is in `.omo/evidence/gui-webui-tauri/task-2-refactor-loc.txt`; test evidence is in `.omo/evidence/gui-webui-tauri/task-2-refactor-tests.txt`.
