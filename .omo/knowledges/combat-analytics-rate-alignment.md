# Combat Analytics rate alignment

The WebUI Combat Analytics `KILL RATE / HR` and `SCAN RATE / HR` must match the CLI status bar, not the recent 10-minute rate fields.

- CLI source of truth: `src/terminal.rs` uses `SessionState::total_kill_rate_per_hour_at(now)` and `SessionState::total_scan_rate_per_hour_at(now)`.
- Session formulas live in `src/state.rs`: total rates are `events / elapsed session time * 1 hour`; recent rates use `RECENT_RATE_WINDOW` and are intentionally different.
- WebUI source: `ui/src/components/dashboard/tactical-telemetry-view.tsx` should render `snapshot.session.kill_total_rate_per_hour.display` and `snapshot.session.scan_total_rate_per_hour.display`.
- CLI-compatible zero-event display is `-/h`, so WebUI overrides the backend `0.0/h` display when the matching event counter is zero.
- Total rates can change as elapsed time changes even without new events. `ui/src/store/dashboard-store.ts` therefore performs a silent periodic `loadSnapshot()` while a live subscription is active, so rate-only snapshot updates are applied.
- Regression coverage: `ui/e2e/combat-analytics.spec.ts` checks total-vs-recent display, zero-event `-/h`, and live refresh of total rates.
