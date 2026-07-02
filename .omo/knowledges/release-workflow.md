# Release workflow

- The root `Cargo.toml` package version is the only manual release version source.
- `scripts/sync-release-version.mjs --check-tag vX.Y.Z` verifies the tag and syncs `ui/src-tauri/Cargo.toml` plus `ui/src-tauri/tauri.conf.json` before release builds.
- GitHub Release assets are versioned for smartrelease matching:
  - `ed-sentry-vX.Y.Z-windows-x64.zip`
  - `ed-sentry-vX.Y.Z-linux-x64.zip`
- README download buttons use bytedream smartrelease patterns and keep the GitHub Releases page as a fallback.
- Windows packages include `ed-sentry.exe`, `ed-sentry-core.exe`, `WebView2Loader.dll`, `config.toml`, `README.md`, `LICENSE`, `webui/`, and `tools/cloudflared/`.
- Linux packages include `ed-sentry-core`, `config.toml`, `README.md`, `LICENSE`, and `webui/`.

## v0.1.0 release verification, 2026-07-02

- `v0.1.0` was moved to `5d5214974d5aaf65e38591d71c8395b83a9bb8b0` after `master` CI passed.
- Release workflow run `28562493836`, attempt `2`, completed successfully.
- GitHub Release `ED Sentry v0.1.0` is published, not draft, not prerelease.
- Published assets:
  - `ed-sentry-v0.1.0-windows-x64.zip`
  - `ed-sentry-v0.1.0-linux-x64.zip`
  - `checksums.txt`
- README smartrelease URLs downloaded both zip files successfully.
- Downloaded SHA-256 values matched `checksums.txt`:
  - Windows: `ae017ec60bd7a6f4dfff652b50526bcd4a704126c2e2f4426ce5f83f0dd8bdd9`
  - Linux: `327a48c2e55b91d5fcb7e15389973578caebb7148289ded3163df7e755e02a1c`
- Downloaded Windows zip contains `ed-sentry.exe`, `ed-sentry-core.exe`, `WebView2Loader.dll`, `config.toml`, `README.md`, `LICENSE`, `webui/index.html`, and `tools/cloudflared/cloudflared.exe`.
- Downloaded Linux zip contains `ed-sentry-core`, `config.toml`, `README.md`, `LICENSE`, and `webui/index.html`.
