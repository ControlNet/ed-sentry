# Desktop GUI Phase One

The phase-one desktop GUI entry is `ed-sentry-gui`, a Tauri v2 app under
`ui/src-tauri/` that reuses the shared `ui/` dashboard frontend and the Rust
monitor runtime services from the root crate.

Desktop artifacts are locally buildable with `pnpm --dir ui tauri:build` or the
direct Tauri command form `pnpm --dir ui tauri build`.
This milestone does not add CI packaging or release-upload jobs for desktop
artifacts. The tracked blocker is desktop runner coverage and platform-specific
Tauri packaging/release upload for each supported OS.

The tag release workflow publishes CLI/WebUI archives only. Those archives build
`ui/dist` and place it in `webui/` beside the packaged `ed-sentry` executable so
the phase-one non-embedded asset lookup works without a repo checkout.

The local Windows GNU package includes Cloudflare Quick Tunnel support at
`ed-sentry/tools/cloudflared/cloudflared.exe` with the Apache-2.0 license staged
beside it as `ed-sentry/tools/cloudflared/LICENSE-cloudflared.txt`. Desktop
Tauri tunnel start and config save paths stay local native commands, so they do
not require tunnel login or browser Bearer tokens.
