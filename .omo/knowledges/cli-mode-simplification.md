# CLI mode simplification

- The CLI intentionally has no mode subcommands. Default invocation is watch mode.
- Pass top-level `--replay` to read the selected Journal file from start to finish and exit.
- Replay requires `--set-file`, rejects `--journal`, and rejects `--poll-interval-ms` so no replay-only no-op flag is accepted.
- Watch examples should use `ed-sentry --journal <folder>` or `ed-sentry --set-file <file>`, not `ed-sentry watch ...`.
- Replay examples should use `ed-sentry --replay --set-file <file>`, not `ed-sentry replay ...`.
