# Windows Packaging Rule After UI Changes

The user explicitly asked not to wait for a reminder before rebuilding the Windows zip. After any frontend, Tauri, WebUI asset, or packaging-related change, run:

```bash
./scripts/package-windows-gnu.sh
```

The expected artifact is `dist/ed-sentry-x86_64-pc-windows-gnu.zip`. The staging folder `dist/ed-sentry/` should include `ed-sentry.exe`, `ed-sentry-gui.exe`, `WebView2Loader.dll`, `config.toml`, and `webui/index.html`.

`reference-design/design1.tsx` is a read-only style reference and must not be modified when aligning the implementation to it.
