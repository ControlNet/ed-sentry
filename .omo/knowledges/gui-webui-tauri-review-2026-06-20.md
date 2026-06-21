# GUI/WebUI/Tauri plan review - 2026-06-20

- High-accuracy review verdict was `ITERATE`, not `REJECT`: the architecture and user decisions are sound, but the original plan needed more concrete execution details before implementation.
- Two independent review sources were used: Momus plan reviewer and `codex exec` read-only review. Momus evidence is `.omo/evidence/gui-webui-tauri/review/momus-plan-review.md`; CLI review output is `.omo/evidence/gui-webui-tauri/review/codex-plan-review.txt`.
- The revised plan adds hardening decisions for `ConfigSource`/`ConfigPath`, comment-preserving TOML writes, loopback-only config mutation, WebUI asset lookup/packaging, Tauri `ui/src-tauri` layout, dependency feature requirements, Playwright setup, WebSocket smoke probing, final verification, and secret/privacy scans.
- The selected first-milestone asset strategy is not binary embedding: package built `ui/dist` as sibling `webui/`, with lookup order `ED_SENTRY_WEBUI_DIST`, sibling `webui/`, then repo-local `ui/dist`.
- First-milestone WebUI config mutation is loopback-only. Non-loopback WebUI binds may expose read-only status/snapshot with warning, but state-changing endpoints must reject with `403` until an authenticated remote mode is explicitly designed.
- Tauri implementation target is `ui/src-tauri/`, using shared `ui/` pages and path dependency on the root `ed-sentry` library crate. The desktop product/binary name remains `ed-sentry-gui`.
