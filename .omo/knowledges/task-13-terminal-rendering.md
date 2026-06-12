# Task 13 Terminal Rendering Knowledge

- `src/terminal.rs` owns display only. Monitor code continues to emit `Notification`; `TerminalNotifier<W: Write>` implements `Notifier` and formats those notifications for a writer.
- Use `TerminalNotifier::plain(writer, TimeDisplayZone::Utc)` for deterministic tests and non-TTY output. Plain mode uses `writeln!` and never emits crossterm clear-line/control sequences.
- Use `TerminalNotifier::stdout(&MonitorConfig)` for runtime stdout selection. It chooses TTY mode only when `monitor.live_status` is true and stdout is a terminal; otherwise it stays plain.
- `render_status_line(&SessionState, &MonitorConfig, now)` is deterministic from state/config/time. It currently renders kills, total kill rate, scans, last kill duration, mission completed/total, and shield state.
- `src/text.rs::line_safe` replaces embedded newlines, carriage returns, and control bytes with spaces. This is the shared guard for plain terminal logs/status text and prevents notification text from injecting ANSI controls.
- Task 13 evidence files are `.omo/evidence/task-13-status.txt` and `.omo/evidence/task-13-non-tty.txt`.
