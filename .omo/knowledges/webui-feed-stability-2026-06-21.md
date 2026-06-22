# WebUI Feed Stability Fix - 2026-06-21

## Context

Dashboard browsing became unstable with real WebUI data because live snapshots and event history were entering the frontend through multiple paths.

## Findings

- WebSocket `hello` already includes a full `snapshot` whose `event_feed` contains recent history. Expanding the same `hello.event_feed` into separate frontend `event` updates causes redundant state updates during bootstrap.
- Backend snapshots can be published frequently for runtime status and generated timestamps. Replacing the full frontend snapshot for volatile-only changes causes unnecessary dashboard re-render churn.
- Backend event history is oldest-first and can contain up to 200 entries. Rendering it directly makes the page very long and puts the newest event at the bottom.
- `journal.folder = ""` is valid backend semantics: Rust journal discovery resolves an empty folder through the default Windows Saved Games path. The GUI must not require it.

## Fix Pattern

- Treat WebSocket `hello` as one bootstrap snapshot on the frontend.
- Normalize snapshot event feeds at the store/component boundary: newest first, deduped by existing event merge path, capped to 30 visible items.
- Ignore snapshot updates when only volatile fields such as `generated_at` or event feed ordering changed; still apply snapshots when session, missions, journal source, matrix, or web status changes.
- Keep `Recent event feed` internally scrollable instead of allowing it to grow the whole dashboard page.
- Let an empty Journal folder save through the config UI so backend defaults remain usable.

## Verification

- `pnpm --dir ui typecheck`
- `pnpm --dir ui lint`
- `pnpm --dir ui test:e2e`
- `scripts/package-windows-gnu.sh`
- Visual evidence: `.omo/evidence/gui-webui-tauri/event-feed-long-bounded.png`

## Packaging Rule

After WebUI changes that affect user-visible behavior, rebuild the Windows distributable zip without waiting for a separate reminder. The expected local command is `scripts/package-windows-gnu.sh`, and the expected artifact is `dist/ed-sentry-x86_64-pc-windows-gnu.zip`.

The Windows zip must include both entry points:

- `ed-sentry.exe`: CLI/watch executable with packaged sibling `webui/` assets.
- `ed-sentry-gui.exe`: Tauri desktop GUI executable.
- `WebView2Loader.dll`: required beside `ed-sentry-gui.exe`; otherwise Windows reports that code execution cannot proceed because `WebView2Loader.dll` was not found.
