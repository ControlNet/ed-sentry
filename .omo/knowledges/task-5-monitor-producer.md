# Task 5 Monitor Producer Refactor

- `EventMonitor` is a synchronous producer-only state machine: construct it with `EventMonitor::new(monitor_config, log_levels)` or `EventMonitor::from_runtime_config(config)`.
- `process_event` mutates state first, preserves warning reset/kill recording order, then returns `Vec<Notification>` for caller-owned delivery.
- `check_warnings_at` returns `Vec<Notification>` with current behavior represented as an empty or one-element vector; `start_monitor` returns a single `Notification`; `finish` returns summary/stopped notifications as `Vec<Notification>`.
- `NotificationDispatcher` remains the delivery boundary and still filters level `0`, while level `0` notifications may be returned by the producer for routing decisions.
