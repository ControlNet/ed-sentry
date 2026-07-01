# Release workflow

- The root `Cargo.toml` package version is the only manual release version source.
- `scripts/sync-release-version.mjs --check-tag vX.Y.Z` verifies the tag and syncs `ui/src-tauri/Cargo.toml` plus `ui/src-tauri/tauri.conf.json` before release builds.
- GitHub Release assets are versioned for smartrelease matching:
  - `ed-sentry-vX.Y.Z-windows-x64.zip`
  - `ed-sentry-vX.Y.Z-linux-x64.zip`
- README download buttons use bytedream smartrelease patterns and keep the GitHub Releases page as a fallback.
- Windows packages include `ed-sentry.exe`, `ed-sentry-core.exe`, `WebView2Loader.dll`, `config.toml`, `README.md`, `LICENSE`, `webui/`, and `tools/cloudflared/`.
- Linux packages include `ed-sentry-core`, `config.toml`, `README.md`, `LICENSE`, and `webui/`.
