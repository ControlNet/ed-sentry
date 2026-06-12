# Task 15 README Documentation Decisions

- `README.md` is the Phase 1 user-facing source for local CLI usage. It presents `ed-afk-monitor` as an independently implemented Elite Dangerous Journal AFK monitor, not a fork, port, or copy.
- The README must keep Matrix in deferred Phase 2 language only. Phase 1 has notifier abstractions, but no Matrix homeserver, token, room, command, or delivery configuration.
- Required Phase 1 commands are documented exactly: `cargo run -- --journal "/path/to/Elite Dangerous"`, `cargo run -- --replay --set-file tests/fixtures/journal_combat_bounty.log --no-status-line`, `cargo test --all`, and `cargo test --test real_journal_replay -- --ignored`.
- `config.example.toml` can carry Phase 1 comments, but the `[journal]`, `[monitor]`, and `[log_levels]` defaults now track the upstream-style output contract captured in `src/config.rs` and `.omo/knowledges/output-format-alignment.md`, not the original stale plan values.
- Raw local Journals are read-only inputs and must not be committed. Committed fixtures stay synthetic and sanitized under `tests/fixtures/`.
