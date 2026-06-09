# Task 4 Time And Statistics Primitives

- `src/time.rs` owns deterministic time primitives: `Clock`, `SystemClock`, `FixedClock`, `format_timestamp`, `TimeDisplayZone`, and `format_duration`.
- Use `FixedClock` or explicit `DateTime<Utc>` values in monitor/state tests; avoid `Utc::now()` in warning and rate tests.
- `TimeDisplayZone::FixedOffset` is the deterministic local-style display hook for tests; `TimeDisplayZone::Local` is available for runtime system-local rendering.
- `src/state.rs` exposes total kill/scan rates and recent kill/scan rates. Total rates require `Option<DateTime<Utc>>` session start and return `0.0` for inactive sessions.
- Recent rates use an inclusive 10-minute rolling window ending at `now`; timestamps before `now - 10 minutes` or after `now` are ignored.
