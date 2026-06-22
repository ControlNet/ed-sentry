## Project Rules

- Code, comments, and identifiers are English. Reply to the user in their language unless they ask otherwise.
- Prefer minimal, reviewable patches and do not revert user changes.
- For non-trivial changes, include exact verification commands and expected pass signals.
- Never use `git clean`.
- Never include secrets, tokens, or keys in code, logs, docs, or examples.
- Do not modify `reference-design/design1.tsx`; it is a read-only design reference.
- When frontend, Tauri, packaging, or WebUI asset files change, rebuild the Windows artifact before reporting completion:

```bash
./scripts/package-windows-gnu.sh
```

Expected signal: `dist/ed-sentry-x86_64-pc-windows-gnu.zip` is regenerated, and the script prints SHA-256 lines for `ed-sentry.exe`, `ed-sentry-gui.exe`, `WebView2Loader.dll`, and `webui/index.html`.
