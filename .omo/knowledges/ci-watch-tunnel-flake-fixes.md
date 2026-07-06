# CI watch and tunnel fixture flake fixes

## Context

GitHub CI exposed two platform/timing issues after the release workflow changes:

- `tests/cli_config_watch.rs::cli_config_watch_preloads_existing_event_output` failed on Ubuntu because `capture_watch_startup` returned after startup plus the first `Scan:` line, before the later `Kill:` line had necessarily been read.
- `tests/tunnel_lifecycle.rs` failed on Windows because the fake `cloudflared` fixture was a Unix shell script. Windows `Command::new` needs a Windows-runnable fixture such as `.cmd`.

## Fix pattern

- When a watch test asserts event output beyond startup, wait for the exact asserted fragments with `capture_watch_output_until` instead of the generic startup helper.
- Keep tunnel lifecycle fixtures platform-aware: generate Unix shell scripts on Unix and `.cmd` scripts on Windows.
- Keep provider-level fake `cloudflared` fixtures platform-aware too; Windows cannot execute `#!/bin/sh` fixtures via `Command::new`, and `.cmd` output/log assertions should compare logical lines rather than raw LF bytes.
- Keep Windows batch output-drain fixtures large enough to fill the child pipe but small enough to reach their completion marker inside CI deadlines. A 200,000-line `.cmd` loop can exceed a 2-second marker wait even when readers drain correctly; 2,000 lines is enough for this contract.
- Avoid infinite `.cmd` loops in Windows process fixtures. Finite post-URL waits keep the child alive for provider/lifecycle ownership tests without risking a stuck GitHub Actions test step if Windows process-tree cleanup behaves differently.
- WebUI and desktop fake `cloudflared` fixtures need the same Windows treatment as provider/lifecycle fixtures. Shared WebUI tunnel fixtures should generate `.cmd` on Windows, keep post-URL waits finite, and live outside large test support modules so each test file stays reviewable.
- Avoid racing child exit polling against stdout/stderr reader tasks when the desired contract is “URL reported before output closes”. Let the output channel decide success/failure, then query the child exit status only after the channel closes without a URL.
- Matrix fake-delivery assertions can lag behind terminal stdout on Windows CI. Use a dedicated remote-delivery record deadline instead of reusing the shorter terminal readiness deadline, and include observed fake records in timeout diagnostics.
- WebUI TOML test fixtures must write Windows paths with TOML escaping, not raw `"{}"` interpolation. Use `folder = {:?}` with `path.display().to_string()` so backslashes become valid TOML string escapes instead of invalid `\U` Unicode sequences.
- For lifecycle tests that need to observe a tunnel as `Running` before a crash, do not use a fake `cloudflared` that prints the URL and exits immediately. Gate the fake process exit on an explicit test signal file, then poll `refresh()` with a bounded timeout until it reports `Error`; otherwise CI can race child exit/status polling against stdout reader delivery and fail the initial `Running` assertion.
- `tests/tunnel_lifecycle.rs` also spawns fake `cloudflared` processes and can fail under parallel Rust test scheduling. Mirror `tests/tunnel_provider.rs`: guard all lifecycle tests that spawn or assert non-spawn of a fake executable with a shared async mutex.
