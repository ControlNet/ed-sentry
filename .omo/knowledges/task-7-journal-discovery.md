# Task 7 Journal Discovery And Preload

- `src/journal.rs` now owns journal file discovery and selection. It discovers only `Journal.*.log` files, resolves explicit `journal.folder` first, and otherwise derives the Windows default folder from `USERPROFILE/Saved Games/Frontier Developments/Elite Dangerous`.
- Newest-first ordering uses parsed filename timestamps for both `Journal.YYMMDDHHMMSS.01.log` and `Journal.YYYY-MM-DDTHHMMSS.01.log`; files with unparsed timestamp segments fall back to filesystem mtime, with path ordering as the final deterministic tie-breaker.
- `recent_journal_file_choices` returns deterministic 1-based numeric choices capped by `journal.recent_files`; `select_configured_journal_file` returns `RuntimeConfig.set_file` directly before attempting discovery.
- `preload_journal_file` and `preload_journal_file_with_options` read to EOF using a parser callback, return per-line parse results and the EOF byte offset, and do not know about or dispatch notifications. The options/result flag is the Task 9 hook for clearing counters after preload when `--reset-session` is active.
