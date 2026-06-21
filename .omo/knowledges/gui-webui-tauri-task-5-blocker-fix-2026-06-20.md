# GUI WebUI Tauri Task 5 Blocker Fix

Date: 2026-06-20
Worktree: `/home/ubuntu/.herdr/worktrees/ed-afk-monitor/feature-ui`

Task 5 gate blockers were fixed at the app DTO boundary.

## Facts

- `JournalSourceView.selected_file` must be display-safe metadata, not a private absolute Journal path.
- `src/app/feed.rs` now uses `selected_file_display(&Path)` for config/feed DTOs. It returns a `line_safe` file basename or `<selected Journal file>` when no basename exists.
- `src/app/runtime.rs` now uses the same `selected_file_display` helper for live runtime snapshot `journal_source.selected_file`.
- `src/app/missions.rs` now sanitizes Journal-derived mission DTO strings with `line_safe`:
  - mission display name
  - issuing faction
  - target faction
  - destination system
  - destination station
  - massacre progress target and target faction
  - trade progress commodity display
- `tests/runtime_service.rs` now has failing-first coverage for both classes:
  - DTO JSON must not contain the full temp Journal path.
  - Mission-derived text with newline and ANSI/control content must be line-safe before serialization.

## Evidence

- Failing-first: `.omo/evidence/gui-webui-tauri/task-5-failing-first.txt`
- Fixed focused test: `.omo/evidence/gui-webui-tauri/task-5-runtime-service-focused.txt`
- Code review: `.omo/evidence/gui-webui-tauri/task-5-code-review.md`
- Manual QA matrix: `.omo/evidence/gui-webui-tauri/task-5-manual-qa-matrix.md`
- Full test suite: `.omo/evidence/gui-webui-tauri/task-5-cargo-test-all.txt`
- Clippy: `.omo/evidence/gui-webui-tauri/task-5-cargo-clippy.txt`

## Verification Summary

All requested Rust gates passed:

- `cargo fmt --check`
- `cargo test --test runtime_service runtime_service_emits_sanitized_snapshot_and_notifications_from_fixture`
- `cargo test --test replay`
- `cargo test --test cli_config cli_config_watch_tails_until_stopped`
- `cargo test --test live_tail live_tail_temp_file_drives_monitor_notifier_pipeline_without_sleeping`
- `cargo test --all`
- `cargo clippy --all-targets --all-features -- -D warnings`

Grep checks for direct main-loop constructors and unsafe selected_file/mission mappings returned no matches. Pure LOC checks kept every checked app/runtime/test file under 250 pure LOC.
