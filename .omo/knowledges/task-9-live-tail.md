# Task 9 Live Tail Implementation

- `src/journal.rs` exposes `LiveTail::from_preload(path, &preload)` and `LiveTail::from_offset(path, offset)`. Watch mode now prints matching preloaded events through `EventMonitor`, then starts live processing from `preload.eof_offset` so appended lines are not duplicated.
- `LiveTail::poll(parser)` is the one-step polling API. It performs no sleeping and no notification dispatch; callers own the loop and should sleep for `live_poll_interval(&runtime_config)` between ticks.
- Complete lines are split only on `\n`; `trim_line_ending` removes LF and optional CR so both LF and CRLF Journal lines parse identically. A trailing partial line remains in the tail buffer until a future tick completes it with newline.
- Invalid UTF-8 in a complete live line returns a `JournalLineError` record containing `invalid UTF-8` and the start byte offset. The file offset still advances to EOF so later valid lines continue to process.
- If file length becomes smaller than the live offset, `LiveTail::poll` emits `LiveTailWarning::FileTruncated`, clears any partial buffer, and resets the live offset to the current EOF. Rotation remains intentionally out of scope.
