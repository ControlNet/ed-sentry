# CI watch and tunnel fixture flake fixes

## Context

GitHub CI exposed two platform/timing issues after the release workflow changes:

- `tests/cli_config_watch.rs::cli_config_watch_preloads_existing_event_output` failed on Ubuntu because `capture_watch_startup` returned after startup plus the first `Scan:` line, before the later `Kill:` line had necessarily been read.
- `tests/tunnel_lifecycle.rs` failed on Windows because the fake `cloudflared` fixture was a Unix shell script. Windows `Command::new` needs a Windows-runnable fixture such as `.cmd`.

## Fix pattern

- When a watch test asserts event output beyond startup, wait for the exact asserted fragments with `capture_watch_output_until` instead of the generic startup helper.
- Keep tunnel lifecycle fixtures platform-aware: generate Unix shell scripts on Unix and `.cmd` scripts on Windows.
- Keep provider-level fake `cloudflared` fixtures platform-aware too; Windows cannot execute `#!/bin/sh` fixtures via `Command::new`, and `.cmd` output/log assertions should compare logical lines rather than raw LF bytes.
- Avoid racing child exit polling against stdout/stderr reader tasks when the desired contract is “URL reported before output closes”. Let the output channel decide success/failure, then query the child exit status only after the channel closes without a URL.
- Matrix fake-delivery assertions can lag behind terminal stdout on Windows CI. Use a dedicated remote-delivery record deadline instead of reusing the shorter terminal readiness deadline, and include observed fake records in timeout diagnostics.
