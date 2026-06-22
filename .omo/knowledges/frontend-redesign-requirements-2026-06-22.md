# Frontend redesign requirements for another agent

This document is content and behavior requirements only. It intentionally does not prescribe visual style.

## Product context

- Project: `ed-sentry`, an Elite Dangerous AFK monitoring tool.
- Existing runtime modes: CLI watch/replay, optional Matrix delivery, optional WebUI, and separate Tauri desktop GUI.
- The frontend is one shared React/Vite/TypeScript application under `ui/`.
- The same frontend should work in browser WebUI and Tauri desktop GUI through adapter boundaries.
- The browser WebUI is a full-stack local system: Rust backend reads Journal files, owns config writes, maintains current runtime state, and streams events to the frontend.
- The Tauri GUI is not only a frontend wrapper. It should bootstrap the same Rust app services and honor config-enabled `[web]` and `[matrix]` services where supported.
- Existing CLI behavior must remain intact. CLI replay stays terminal-only and should not become a GUI replay mode.

## Non-goals

- No GUI replay view.
- No historical database or multi-day analytics in the first milestone.
- No public/authenticated remote dashboard mode in the first milestone.
- No Matrix command handling.
- No Discord/Telegram/EDMC integrations.
- No automation, key simulation, auto relog, or game control.
- No chart library requirement in the first milestone.
- No raw Journal line viewer.
- No raw Matrix token display.
- No raw private commander/chat content in frontend fixtures, screenshots, logs, or UI state.

## Runtime and data model assumptions

- Backend source of truth is the Rust application service, not frontend memory.
- Backend provides:
  - Current `AppSnapshot`
  - Recent fixed-size backend event buffer
  - Live event stream through WebSocket or equivalent adapter
  - Sanitized config view
  - Config update endpoint/command
- New subscribers should receive current snapshot plus recent buffered events before live events.
- The frontend receives already-sanitized DTOs and should still treat text defensively.
- DTOs often provide both raw values and display strings. The UI should prefer raw values for calculations/sorting and display strings for user-facing labels.

## Required top-level content areas

Top-level navigation can be simple, but the product content is more than two flat pages.

- Dashboard / Overview
- Missions
  - May be its own top-level route, or a strong drill-down area inside Dashboard.
  - Must include both mission list and mission detail surfaces.
- Event Feed / Alerts
  - May be a dashboard panel plus an expanded surface.
- Config
- Service / Source Status surfaces
  - These can be panels inside Dashboard/Config rather than separate routes.

## Dashboard / Overview requirements

The overview should answer: "Is the AFK monitor running, is the game/session healthy, and is anything urgent happening?"

Display:

- Connection state between frontend and backend adapter.
- Snapshot generation time / last update time.
- Commander name when known.
- Ship name/type when known.
- Current star system when known.
- Game mode when known.
- Session active/inactive state.
- Session start time.
- Session elapsed duration.
- End time when applicable.
- Shield status.
- Ship hull percentage/status.
- Fighter alive/dead/unknown state.
- Fighter hull percentage/status when available.
- Kills.
- Scans.
- Bounty total.
- Merits.
- Merits to report.
- Kill total rate per hour.
- Kill recent rate per hour.
- Scan total rate per hour.
- Scan recent rate per hour.
- Last kill timestamp/display.
- Last scan timestamp/display.
- Current warning/degraded/error indicators.
- Mission summary: active count, completed count, total count, status label.
- Journal source summary.
- Matrix service summary.
- Web service summary.
- Recent event feed summary.

Behavior:

- Live updates must not cause visible full-dashboard churn when only volatile timestamp/feed-order fields change.
- The page must remain browsable while events arrive.
- Long event or mission lists must not make the whole page unusably long.
- Empty states should clearly distinguish "not connected", "connected but no data", and "feature disabled".
- Do not expose raw Journal JSON or raw private local file contents.

## Missions requirements

Missions must be treated as first-class content, not only as a compact dashboard table.

### Mission list

The mission list should show all currently tracked missions and allow selecting a mission for detail.

Display for each mission:

- Mission ID.
- Mission display name.
- Mission kind: `massacre`, `trade`, or `other`.
- Kind label.
- Mission state: `active`, `redirected`, `completed`, `failed`, or `abandoned`.
- State label.
- Issuing faction.
- Target faction when available.
- Destination system when available.
- Destination station when available.
- Accepted time.
- Expiry time when available.
- Reward.
- Progress summary.

List-level display:

- Active count.
- Completed count.
- Total count.
- Status label.
- Empty state when no tracked missions exist.

Useful behavior:

- Select/open mission detail.
- Preserve selection when live updates arrive if the mission still exists.
- Show stale/missing values as unknown, not as zero unless zero is semantically true.
- Sort/filter can be discussed, but active/urgent/expiring missions should be easy to find.

### Mission detail

Mission detail is required, especially for trade/collection/delivery missions.

Common mission detail content:

- Mission ID.
- Display name.
- Kind and kind label.
- State and state label.
- Accepted time.
- Expiry time.
- Issuing faction.
- Target faction.
- Destination system.
- Destination station.
- Reward.
- Progress details.
- Related recent events if available from event feed.

Backend model can support more detail than current frontend DTO exposes. If the redesign wants full detail, ask backend to expose:

- Origin system.
- Origin station.
- Origin market ID.
- Wing mission flag.
- Influence.
- Reputation.
- Donation.
- Fine.
- Old/new destination from redirects.
- Completed/failed/abandoned timestamps.
- State history or lifecycle event history.

### Massacre mission detail

Display:

- Target faction.
- Target type.
- Current kills.
- Required kill count.
- Remaining kills.
- Progress display / percent.
- Issuing faction credited by progress.
- Last related kill/bounty event if available.
- Stacked mission behavior if multiple missions can progress from the same bounty context.

Important domain behavior:

- Massacre progress is tracked from relevant bounty events.
- The tracker uses a stack-like rule: one matching active mission per issuing faction progresses for a valid bounty.
- Invalid bounty-like records such as zero reward, suit targets, `faction_none`, and generic pirate sentinels are ignored.

### Trade / collection / delivery mission detail

This is the strongest reason mission detail is needed.

Display:

- Commodity.
- Localized commodity name when available.
- Required count / total count.
- Collected count.
- Delivered count.
- Remaining to collect.
- Remaining to deliver.
- CargoDepot progress.
- Cargo type / localized cargo type when available.
- Destination system.
- Destination station.
- Accepted time.
- Expiry time.
- Related `CargoDepot` events when available.
- Related delivery/completion/failure events when available.
- Redirect old/new destination if backend exposes it.

Current frontend DTO already exposes a trade progress summary:

- `commodity`
- `collected`
- `delivered`
- `count`
- `display`

But a complete detail surface may require DTO expansion.

## Event feed / alerts requirements

The event feed should be readable during long-running monitoring sessions.

Display:

- Timestamp.
- Event source.
- Event type.
- Severity/level.
- Summary.
- Notification text when available.
- Mention flag when available.
- Connection lifecycle events.
- Warning events.
- Matrix delivery/status events.
- Journal source events.
- Mission lifecycle events.
- Mission progress events.
- Kill/scan/bounty events.

Behavior:

- Newest events should be easy to see without forcing users to scroll the full page to the bottom.
- The feed should have an internal browsing model for long sessions.
- Late-opening WebUI/Tauri views must show buffered current-process events from the backend.
- The feed must never display raw Matrix token values, raw Journal lines, or private chat content.
- Event text must be line-safe and frontend-safe.

## Journal source requirements

Display:

- Journal folder.
- Selected/current Journal file.
- Recent files count.
- Source status label.
- Last known read/tailing state if exposed.
- Missing folder/file state.
- Permission error state.
- Stale source state.

Behavior:

- Journal folder is not a required user input because a default exists.
- Show default/resolved folder distinctly from user-configured folder if backend exposes that distinction.
- Paths should be treated as local private data and not copied into public logs/screenshots unnecessarily.
- Browser WebUI cannot directly read local files; backend owns file access.
- Tauri may use native file/folder dialogs through adapter-specific behavior, but shared pages should remain portable.

## Matrix status requirements

Display:

- Matrix enabled/disabled.
- Homeserver.
- User ID if exposed.
- Room ID.
- Mention user ID if configured.
- Delivery status.
- Last delivery result/message.
- Status update interval.
- Checked time.
- Token-present state.

Behavior:

- Never display, log, store, or echo the raw Matrix access token.
- Token editing is write-only.
- The frontend may allow token replacement and token clearing through explicit fields/actions.
- Matrix is optional and best-effort; Matrix failures should not stop monitoring.

## Web service status requirements

Display:

- Web enabled/disabled.
- Host.
- Port.
- Open-browser setting.
- Bind address.
- URL.
- Running/warning/error status.
- Checked time.
- Startup failure message if backend exposes one.

Behavior:

- `[web] enabled = true` starts the WebUI in watch-capable runtime modes.
- No separate `--webui` flag is required for normal use.
- WebUI defaults to local-first behavior.
- Non-localhost bind may be read-only for state-changing endpoints unless a later authenticated remote mode is designed.

## Config requirements

The GUI should support config editing, not only viewing.

Editable groups:

- Journal:
  - Folder.
  - Recent files.
- Monitor:
  - UTC display setting.
  - Live status setting.
  - Dynamic title setting.
  - Kill rate warning threshold.
  - Kill rate delay.
  - No-kill warning threshold.
  - Initial no-kill warning threshold.
  - Warning cooldown.
  - Duplicate suppression maximum.
  - Pirate names.
  - Bounty faction display.
  - Bounty value display.
  - Extended stats.
  - Minimum scan level.
  - Poll interval.
- Log levels:
  - All configured log level keys, including mission-related `missions` and `missions_all`.
- Matrix:
  - Enabled.
  - Homeserver.
  - User ID.
  - Room ID.
  - Mention user ID.
  - Status update interval.
  - Access token replacement.
  - Clear access token.
- Web:
  - Enabled.
  - Host.
  - Port.
  - Open browser.

Config policy/status content:

- Config version.
- Whether state-changing config updates are enabled.
- Reason config updates are enabled/disabled.
- Whether current bind is remote.
- Save state: clean, dirty, saving, saved, validation error, permission error.
- Permission/read/malformed config errors in frontend-safe wording.

Security behavior:

- Matrix token is write-only.
- Backend should preserve unknown TOML keys/comments where supported.
- State-changing config endpoints should be loopback-only in the first milestone.
- Do not expose committed/local `config.toml` content in screenshots or docs.

## Adapter requirements

The shared frontend should depend on a dashboard adapter interface, not hardcoded environment APIs.

Supported adapter modes:

- `mock`
- `web`
- `tauri`

Adapter responsibilities:

- Load initial snapshot.
- Subscribe to snapshot/event/connection updates.
- Load sanitized config.
- Submit config updates.
- Represent connection state: idle, loading, connected, degraded, error.
- Provide environment-specific details without duplicating pages.

Web adapter:

- Uses HTTP for snapshot/config.
- Uses WebSocket for live events/snapshots.
- Handles malformed payloads as degraded/error states without crashing the UI.

Tauri adapter:

- Uses Tauri-native commands/events where available.
- Still shares the same pages/components/store.
- Desktop entry should honor config-enabled Web/Matrix services through Rust app service behavior.

Mock adapter:

- Uses sanitized fixture-like data.
- Must not contain real Journal data, real commander private data, or secrets.
- Useful for frontend brainstorming, Storybook-like states, Playwright tests, and offline UI review.

## Current frontend DTO fields

Current `AppSnapshot` shape includes:

- `generated_at`
- `generated_at_display`
- `session`
- `missions`
- `notifications`
- `event_feed`
- `journal_source`
- `matrix`
- `web`

Current `SessionView` includes:

- `commander`
- `ship`
- `system`
- `mode`
- `active`
- `status_label`
- `started_at`
- `started_at_display`
- `ended_at`
- `ended_at_display`
- `elapsed_seconds`
- `elapsed_display`
- `shields_up`
- `shields_display`
- `ship_hull_percent`
- `ship_hull_display`
- `fighter_hull_percent`
- `fighter_hull_display`
- `fighter_alive`
- `kills`
- `scans`
- `bounty_total`
- `merits`
- `merits_to_report`
- `kill_total_rate_per_hour`
- `kill_recent_rate_per_hour`
- `scan_total_rate_per_hour`
- `scan_recent_rate_per_hour`
- `last_kill_at`
- `last_kill_display`
- `last_scan_at`
- `last_scan_display`

Current `MissionView` includes:

- `mission_id`
- `state`
- `state_label`
- `kind`
- `kind_label`
- `display_name`
- `issuing_faction`
- `target_faction`
- `destination_system`
- `destination_station`
- `accepted_at`
- `accepted_at_display`
- `expiry`
- `expiry_display`
- `reward`
- `progress`

Current mission progress variants:

- `none`
- `massacre`: `target`, `target_faction`, `kills`, `kill_count`, `display`
- `trade`: `commodity`, `collected`, `delivered`, `count`, `display`

Current `EventFeedItem` includes:

- `id`
- `source`
- `event_type`
- `level`
- `summary`
- `timestamp`
- `timestamp_display`

Current `JournalSourceView` includes:

- `folder`
- `selected_file`
- `recent_files`
- `status_label`

Current generic service status view includes:

- `kind`
- `status_label`
- `message`
- `room_id`
- `bind_address`
- `url`
- `checked_at`
- `checked_at_display`

Current config view includes:

- `journal`
- `monitor`
- `log_levels`
- `matrix`
- `web`
- `policy`

## Known current UI pain points to avoid

- Dashboard should not appear to constantly re-render or visually churn during live updates.
- Recent event feed should not make the full page grow indefinitely.
- Recent/new events should be easy to access.
- Long mission/event content must remain browsable.
- Journal folder should not be treated as a required field with no default.
- Mission content should not be reduced to a compact row-only table; detail is needed.

## Suggested discussion points for the frontend agent

- Whether Missions should be a top-level route or a dashboard drill-down surface.
- How mission detail should be opened: route, split pane, drawer, or modal.
- How to browse long event streams while keeping newest events accessible.
- How to preserve selected mission/detail state through live updates.
- Which mission detail fields require backend DTO expansion before implementation.
- How to represent unknown/unavailable data without misleading zero values.
- How to handle config editing ergonomically while keeping token replacement write-only.
- How to support both WebUI and Tauri without duplicating page code.

