# GUI/WebUI/Tauri Code-Quality Rerun Fix

Date: 2026-06-20

The Global Review rerun blockers were fixed in the feature UI worktree by replacing fixed watch-process sleeps with observable-driven waits, broadening the verifier secret scan to include untracked working-tree files, and removing a production `expect()` from TOML config writing.

Important implementation details:

- `tests/cli_config/capture_output.rs` owns the general watch-process helper. It reads stdout/stderr on threads and waits for explicit stdout/stderr markers or process exit/deadline before stopping the child.
- `tests/cli_config/capture_text.rs` stays self-contained because integration tests compile as separate crates; nesting `capture_output.rs` inside it triggers clippy `duplicate_mod` when a test also imports `capture_output`.
- `tests/cli_config/matrix.rs` has `wait_for_matrix_record` for fake Matrix log observables. This avoids stopping the watch process before async Matrix status records are written.
- `scripts/verify-gui-webui-tauri.sh` now supports `VERIFY_GUI_WEBUI_TAURI_ONLY_SECRET_SCAN=1 bash scripts/verify-gui-webui-tauri.sh` to run the same sanitized working-tree privacy scan without the full GUI/Tauri gate.
- `src/config/write/apply.rs` normalizes TOML sections with an exhaustive `match` loop instead of using `expect()`.

Verification artifacts for this pass use the prefix `.omo/evidence/gui-webui-tauri/global-review-code-quality-rerun-fix-*`.
