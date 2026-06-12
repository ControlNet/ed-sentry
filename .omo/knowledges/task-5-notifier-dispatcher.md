# Task 5 Notifier Dispatcher

- `src/notifier.rs` owns Phase 1 notification abstractions: `AlertLevel`, `Notification`, synchronous `Notifier`, `FakeNotifier`, and `NotificationDispatcher`.
- `Notification::new` sets `mention` from the log level contract with `level >= 3`; level `0` is ignored by the dispatcher.
- Phase 1 does not implement Matrix HTTP or remote delivery. `remote_text` and `mention` are stored for future Matrix work, but dispatch remains terminal/notifier-only.
- Upstream parity decision: Phase 1 terminal delivery does not suppress duplicates. `duplicate_max` and `duplicate_suppression` remain in config for future remote-delivery controls, but `NotificationDispatcher` passes terminal notifications through unchanged.
