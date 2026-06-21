# GUI WebUI Tauri Task 8 Path Redaction

Date: 2026-06-20

Todo 8 gate blocker: `/api/snapshot` and WebSocket `hello` leaked the configured raw Journal folder through `journal_source.folder` when `MonitorRuntime::snapshot` copied `watch_journal_folder_display(&config)` into the frontend DTO.

Fix:

- Keep `watch_journal_folder_display` for CLI/terminal output because existing CLI behavior intentionally displays the local Journal folder.
- Use `snapshot_journal_folder_display` for frontend runtime snapshots:
  - explicit configured folder => `Configured journal folder`
  - default discovery folder => `Default journal folder`
- Keep `journal_source.selected_file` basename-only through `selected_file_display`.
- Do not redact `/api/config` `journal.folder` in this fix; it remains part of the local config editor contract.

Evidence pattern:

- Failing-first smoke must prove the old leak with a real `/tmp/tmp.../journal` grep before the fix.
- Regression tests should drive real HTTP and WebSocket surfaces from a runtime-backed event store, not `JournalSourceView::unknown`.
- Snapshot smoke should wait for the fixture Journal event (`Scan: Smoke Raider`) before fetching `/api/snapshot`; otherwise it can pass against the initial startup snapshot and miss the privacy surface.
- Final artifacts must include negative grep checks over `.omo/evidence/gui-webui-tauri/task-8-snapshot.http` and `.omo/evidence/gui-webui-tauri/task-8-websocket.jsonl`.
