# GUI WebUI Tauri Task 6 Event Buffer

Date: 2026-06-20

Todo 6 added an in-process backend event store in `src/app/events.rs` and integrates it through the app runtime service.

Implementation notes:
- `AppEventStore` owns bounded `VecDeque` histories for `EventFeedItem` and `NotificationView`.
- `AppEventBootstrap` returns the current backend `AppSnapshot` plus recent buffered events.
- `AppLiveUpdate` broadcasts typed live updates as either `Snapshot { snapshot: Box<AppSnapshot> }` or `Event { item: EventFeedItem }`.
- The store uses standard-library fan-out subscribers because Tokio's existing dependency features do not include `sync`, and this task did not edit `Cargo.toml`.
- Sanitization happens at insertion: notification records use existing `NotificationView` / `EventFeedItem` conversions, and lifecycle/status/warning event text uses `line_safe`.
- Runtime warnings and notifications are recorded in the backend event store before snapshots are published.
- Monitor lifecycle is now wired in production through `MonitorRuntime::start_monitor_if_preloaded`, which records `monitor_started`.
- Status feed events are now wired in production through `MonitorRuntime::status_snapshot`, which records `runtime_status` when a status line or dynamic title is rendered.
- `AppSnapshot.notifications` and `AppSnapshot.event_feed` are populated from the backend store, not frontend memory.
- No disk persistence, database, raw Journal lines, Matrix token fields, or private path fields were added to the production event-store DTO/runtime files.

Blocker-fix notes:
- The weak privacy test was strengthened with seeded `raw_payload`, `access_token`, and `/home/private` values in the event-store path, then asserted absent from serialized bootstrap after eviction.
- A runtime malformed-preload test writes a raw Journal line with the same private-looking seeds and proves serialized snapshots expose only the sanitized warning DTO, not the raw line.
- `src/app/tests.rs` delegates Todo 6 event-store bodies into `src/app/tests/event_store.rs` to keep both files under the 250 pure-LOC gate while preserving the original gate test filters.
- `RuntimeBatch::empty` keeps `src/app/runtime/service.rs` under the module-size gate after lifecycle/status wiring.

Verification artifacts:
- Failing-first blocker fix: `.omo/evidence/gui-webui-tauri/task-6-failing-first-fix.txt`
- Required event-buffer filter: `.omo/evidence/gui-webui-tauri/task-6-test-event-buffer-filter.txt`
- Subscriber bootstrap: `.omo/evidence/gui-webui-tauri/task-6-event-buffer.txt`
- Eviction edge: `.omo/evidence/gui-webui-tauri/task-6-event-buffer-eviction.txt`
- New focused runtime/privacy tests: `.omo/evidence/gui-webui-tauri/task-6-focused-new-tests.txt`
- Full suite: `.omo/evidence/gui-webui-tauri/task-6-test-all.txt`
- Clippy: `.omo/evidence/gui-webui-tauri/task-6-clippy.txt`
- Format: `.omo/evidence/gui-webui-tauri/task-6-fmt-check.txt`
- Grep/privacy: `.omo/evidence/gui-webui-tauri/task-6-grep-event-store-dtos.txt`
- Pure LOC: `.omo/evidence/gui-webui-tauri/task-6-pure-loc.txt`
