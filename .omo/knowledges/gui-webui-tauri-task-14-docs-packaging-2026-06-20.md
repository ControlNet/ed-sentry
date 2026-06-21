# GUI WebUI Tauri Task 14 Docs And Packaging

Date: 2026-06-20

## Facts Learned

- WebUI startup is controlled by `[web] enabled = true` in config for watch-capable CLI and desktop runtimes. There is no separate startup switch.
- Replay remains terminal-only and does not initialize the WebUI.
- First-milestone WebUI config mutation is loopback-only. Non-loopback binds may serve read-only status/static surfaces but must reject state-changing config updates.
- WebUI assets are not embedded in the Rust binary. Runtime lookup order is `ED_SENTRY_WEBUI_DIST`, executable sibling `webui/`, then repo-local `ui/dist`.
- Release archives should stage `ed-sentry/ed-sentry` or `ed-sentry/ed-sentry.exe`, `ed-sentry/config.toml`, and `ed-sentry/webui/`.
- `ed-sentry-gui` is locally buildable with `pnpm --dir ui tauri:build`; CI does not publish desktop artifacts in this milestone.

## Verification Artifacts

- Docs grep: `.omo/evidence/gui-webui-tauri/task-14-docs.txt`
- Packaging docs grep: `.omo/evidence/gui-webui-tauri/task-14-packaging-docs.txt`
- Forbidden claims guard: `.omo/evidence/gui-webui-tauri/task-14-forbidden-claims.txt`
- Privacy guard: `.omo/evidence/gui-webui-tauri/task-14-docs-guard.txt`
- Rust tests: `.omo/evidence/gui-webui-tauri/task-14-cargo-test-all.txt`
- Frontend build: `.omo/evidence/gui-webui-tauri/task-14-pnpm-build.txt`
- Packaged root smoke: `.omo/evidence/gui-webui-tauri/task-14-packaged-webui-root.http`
