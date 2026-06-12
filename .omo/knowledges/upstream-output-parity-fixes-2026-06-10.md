# Upstream output parity fixes - 2026-06-10

- Upstream terminal output does not suppress duplicate event lines; duplicate suppression in `afk_monitor.py::logevent()` only applies to Discord delivery. Phase 1 terminal-only `NotificationDispatcher` should deliver repeated terminal notifications unchanged.
- Fuel ETA follows upstream `ReservoirReplenished`: after a previous fuel sample during an active session, estimate remaining time from elapsed seconds and fuel consumed, then append ` (~<duration>)` to `Fuel: N% remaining`.
- Summary average kill duration uses upstream integer truncation of average seconds, not rounding. Compact credit formatting rounds thousands/millions (`60.7k` displays as `61k`).
- Live status line should match upstream shape: `💥 <rate>/h (+<last kill>) [xN] | 📦 <rate>/h (+<last scan>) [xN] | ⏱️ <session> | 🎯 <redirects>/<active missions>`. Do not include the old `Kills`, `Scans`, `Last kill`, or shield fragments.
- Watch-mode `Monitor started` should be timestamped with actual monitor startup time after preload, not the last preloaded Journal timestamp.
- Warning notifications use the upstream warning emoji `⚠️`; replay can check warnings against Journal timestamps after each parsed event.
