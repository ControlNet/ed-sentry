# Elite Dangerous UI language (redesign reference, 2026-11-15)

The user rejected five generic dark/space redesigns because they did not follow the
game's look. These are the visual rules used for the v2 redesign proposals. They are
inspired by the game only; do not ship Frontier logos, font files, or icons, because
ED Sentry is unofficial.

## Colour
- Near-black base with translucent holographic panels: an orange-tinted fill around
  `rgba(255,113,0,.04-.08)`, 1px orange borders and faint scanlines.
- Primary HUD orange is about `#ff7100`. Use a dimmer orange for secondary text and borders.
- Signature interaction: the focused or selected row/tab/tile is a solid orange fill
  with near-black text.
- Blue (`#3fa9f5`-ish) marks "on" indicators, shields and info. Red marks hostile,
  failure and warnings. The game rarely uses green, so a checklist pass reads as blue.

## Type
- Labels and headers are all caps, in a Eurostile-like face. On Google Fonts, Michroma
  (extended) is a good sparing heading face. Saira Semi Condensed and Rajdhani work for UI text.
- Use tabular numerals. Readability against bright backgrounds is a known player
  complaint, so keep contrast and size reasonable.

## Shapes and motifs
- 45-degree chamfered corners, bracket corner marks and header rules with solid end caps.
- A triangle marker on the focused row, segmented block bars and dotted leaders.
- Odyssey-era diagonal hatch accents.

## Layout archetypes
- Cockpit left panel tabs: Navigation / Transactions / Contacts / Target.
- Right panel tabs: Status / Modules / Fire Groups / Ship / Inventory / Functions.
  Tab strips show Q/E shoulder-key hints.
- HUD corners:
  - Comms panel at top left, info/alerts at top right.
  - Target panel at bottom left, elliptical scanner at bottom centre.
  - Ship status at bottom right: shields, SYS/ENG/WEP pips, blue indicator squares for
    mass-lock, landing gear and cargo scoop.
- Other screens:
  - Station services: large tiles.
  - Mission board: list plus a detail pane with reward boxes.
  - Galaxy map: navy space, blue grid, a left options rail and a top search/route bar.
  - Odyssey menus: a left category bar, top search bar and bottom info window.

## Mapping to ED Sentry
- The AFK checklist (hardpoints deployed, ENG pips 0, cargo) maps naturally onto the
  HUD indicator squares.
- Mock data only provides `Pips[1]` (ENG). Do not invent SYS/WEP, shield, hull, heat or
  fuel values in designs.

## Corrections after viewing real screenshots
Real in-game screenshots (mission board, nav panel, right status panel, SRV panel, Codex
commander stats, outfitting, galaxy map display options, cockpit HUD with comms) showed
that the first spec above was over-decorated. The real UI uses:
- **Type:** a wide, squarish Eurostile at regular weight, not a condensed face. Google
  `Saira` with the width axis around 112-118 is a close free match. Michroma is too
  extended. Titles and lists are caps; descriptive sentences use sentence case.
- **Shapes:** almost no chamfers, bracket corners or hatching. Panels and rows are flat
  rectangles. Hatching ("///") appears only in marketing art.
- **Rows:** a translucent dark orange-brown fill (about `rgba(255,110,0,.12-.16)`) with
  2-4px gaps between rows. The selected row is solid orange with dark text.
- **Colour roles:**
  - White is used for screen titles and key names (e.g. "CMDR NICK", "OUTFITTING",
    "MAJOR FACTIONS").
  - Cyan-blue marks current or target items (current system, mission target), with a dark
    blue row fill.
- **Structures:**
  - Tabs are equal-width rectangular blocks; the active tab is solid orange, and a ">"
    marks overflow.
  - Data rows put a dim orange label on the left, an orange value on the right and a 1px
    rule underneath (outfitting "SHIP SPECS").
  - Codex stat cells are dark grey-brown boxes with a white label and an orange value.
  - Headers read "PLACE | CONTEXT" in orange with a dim second half, then a big title,
    with a clock and date at top right. Every menu has a dark-brown "BACK" block at bottom
    left.
  - The galaxy map has a left rail of dark square icon buttons with the active one in
    solid orange, checkbox squares, and a bottom key-hint bar ("PITCH UP [T]").
  - The cockpit HUD comms panel is an outlined box with a header and
    `TO [CMDR NAME]> message` lines; holo panels have faint scanlines and perspective.

## Tooling notes
- `static.wikia.nocookie.net` serves a Cloudflare "Just a moment..." page to a plain curl
  request. With a browser User-Agent, an image Accept header and a fandom Referer it
  returns the images (mostly WebP). Use the wiki API
  (`api.php?action=query&prop=imageinfo&iiprop=url&titles=File:...`) to get the URLs, and
  convert with ImageMagick `convert` before viewing.
- Keep downloaded game screenshots local as references only; they are Frontier's
  copyrighted material and must not be published or committed.

## User feedback on v2 → v3 direction (2026-11-15)
- Do not use orange as a background or dominant tone (v2 A/B rejected for that). The main
  tone must be dark: near-black or deep navy, white text on solid dark panels. Orange is
  reserved for values, the single selected item, primary buttons and title rules.
- The galaxy-map navy space atmosphere is liked, but text must never sit directly on
  starfield or haze. Readability beats decoration, and a full HUD-as-dashboard concept
  was rejected as confusing.
- Content-dense layouts (tabbed panel, service menu, Odyssey sidebar) are preferred.

## Previewing design-canvas boards locally
The canvas runtime (`support.js`) cannot be downloaded, so a minimal stand-in renderer
(supporting `{{ }}`, `sc-if`, `sc-for`, `on*` handlers and helmet) placed next to a copy of
the board is enough for screenshots. Drive it with Playwright from `ui/node_modules/.pnpm`
using `executablePath: '/usr/bin/google-chrome'`. Plain `google-chrome --headless
--screenshot --virtual-time-budget` hangs on boards with infinite CSS animations.

## v4 decisions (2026-11-15)
- Frame: F (title-header style, layout inspired by the in-game Codex menu) with a plain logo (icon + "ED SENTRY | <system>", no
  bordered box). Content: the D3 galaxy-map style panels on every screen. A3/B3/C are dropped.
- Do not blink FAIL indicators; failure state is shown by static red colour only.
- No entry animations that delay content (no scanline sweep, panel fade/flicker, bar fill-from-0, ring segment fill-in, selection wipe, typing reveal). Content appears instantly; looping ambient motion (starfield, grid, orbit, radar sweep, status breathing, bar sheen, cursor, sidebar icon loops) is welcome.
- No keyboard control: ED Sentry is used with the mouse, so designs must not show
  shortcut hints (no Q/E shoulder-key labels, no key-hint bar) and need no key handlers.
- Design boards may use clearly marked fictional demo names (e.g. system "Kessari")
  instead of the "Sanitized ..." mock placeholders; the app shows real Journal names.

## Chosen direction (2026-11-15)
- The user is satisfied with **"G1 + Motion layer"** (`MotionCards.dc.html` on the design
  canvas): F title header with the plain logo, D3 galaxy-map panels, compact stat
  cards in the sidebar with continuously animated line-art icons, and the motion layer.
- This is the reference to implement in `ui/src`, together with the motion rules above:
  loops are fine; FAIL does not blink; there is no page-switch scanline and no
  content-delaying entry animation.

## Snapshot fields the redesigns must not miss (checked 2026-11-15)
`AppSnapshot.session` (see `ui/src/adapters/types.ts`) already carries:
- Ship health: `shields_up` / `shields_display`, `ship_hull_percent`, `fighter_hull_percent`,
  `fighter_alive`.
- Rates: `kill_recent_rate_per_hour` and `scan_recent_rate_per_hour` next to the total-average
  rates.
- Merits and timing: `merits_to_report`, `last_kill_at`, `last_scan_at`.

`missions` has `completed_count` and `total_count`.

There is no time series, so sparklines need a backend addition first (for example a
per-minute kill/scan ring buffer). Do not fake history in designs.

## G1 v2 iteration (2026-11-15)
G1 v2 (`G1v2.dc.html` on the design canvas) keeps G1 + Motion and adds visual data built only from real snapshot fields:
- recent-vs-average rate bars with the configured 20/h warn tick;
- a live since-last-kill counter against the 20-minute no-kills warning;
- ship status and readiness panel;
- mission accept→expiry timelines with Now/ETA markers;
- a Comms session timeline;
- Systems as one scroll with a live-status index;
- About data sources;
- a service status footer.
Derived values (trend %, time to expiry, combined reward, routing counts) are computed from snapshot/config data, not invented.

## Borrow the style, not the names (2026-11-15)
- Game screens are a visual reference only. Never put in-game menu or feature names
  (e.g. "Codex", "Galactic Library", "Galaxy Map") into ED Sentry's UI text, titles,
  labels or aria-labels. Use neutral app wording instead (e.g. "Software info",
  "App emblem").
- All "Codex" wording was removed from the design boards on 2026-11-15. The board
  `DarkCodex.dc.html` was renamed to `DarkHeader.dc.html`.
- Every label must carry information. Do not add decorative category tags or subtitles
  that restate the title (removed from G1 v2: "Desktop · Software", "Software /
  Monitoring", per-page header subtitles, "Log time", "Telemetry link active").
- Never invent text to fill space (rule set by the user, 2026-11-15). Every string must
  come from snapshot/config data, existing app copy, or describe a real behaviour. The
  same fact must not repeat in several places on one screen. If an area is empty, change
  the layout instead of writing filler.
  - Removed for this reason: "Target lock requires valid bounty progress for the
    tracked faction." (it also exists in `ui/src/components/dashboard/tactical-mission-detail.tsx`
    and should go when the design is implemented).
  - Also removed: "negotiating", "session events", a data-sources note that repeated
    the legal text, and repeated session time, last kill, event count and system name.

## UI copy cleanup of the existing GUI (2026-11-15)
The user wants the redesign to fix the existing GUI's invented sci-fi copy. G1 v2 applies
this mapping, and the `ui/src` implementation should follow it:
- Labels: Comms feed → Events; Combat analytics → Combat; Service nodes → Services;
  Local journal / Matrix relay / Web interface → Journal / Matrix / Web UI.
- Recent alerts / All comms → Recent events / All events; Telemetry log → Event log.
- Mission directory → Active missions; Mission intel → Mission details; Tracking uplink ·
  confirmed kills → Kills; Kill stack → Mission kills.
- Systems sections: Local ingestion / Matrix relay protocol / Tunnel gateway / Local API
  gateway / Monitor thresholds / Event routing levels → Journal / Matrix / Tunnel / Web UI /
  Monitor / Event routing. "No token in vault" → "No access token stored"; "Enable Matrix
  broadcasting" → "Enable Matrix".
- Removed:
  - "Target lock requires…"
  - the per-row routing descriptions, which repeated the selected Off/Notify/Mention button
  - "Fly Dangerously, Commander" (a game slogan)
  - the sidebar "Sections" heading
- Loading screen: `loading-screen.tsx` drives a timer-based fake percentage and stage
  names ("ESTABLISHING MATRIX RELAY…") that do not reflect real progress. Replace them
  with an indeterminate looping bar, "Waiting for first snapshot", and the real `detail`
  from the adapter.
- Second pass:
  - Navigation and section numbers are decoration; dropped "01"–"05" and "01 ·" prefixes.
  - "Total kills/scans" → "Kills/Scans". The footer "Uptime" shows the session length
    from `started_at`, so it is now "Session"; the footer no longer shows the version,
    which is already on About.
  - The readiness detail now shows the row's `source` file instead of developer
    conditions such as "Status Flags bit 0x40 is set". That condition read wrongly on a
    failing row.
  - Missions: removed the stack progress, next-expiry and per-faction bars from the
    Stack panel, the type kicker in Mission details, and the accepted/expires subtitles
    on the timeline. All of these repeated another panel on the same screen.
  - Systems: "6 values" (wrong: 9 fields) → "Warn 20/h · no kills 20 min"; "keys" →
    "events"; removed per-group key counts, the "write-only" tag (it repeated the
    placeholder) and the disabled "SSH tunnel (unsupported)" option.
  - Token notes now read "Leave blank to keep the stored token. The stored token is
    never sent to the browser."
  - About: removed the three "Read-only" badges, which repeated the legal sentence.

## Responsive layout (G1 v2, 2026-11-15)
The 1440x900 board broke below about 1300 px. The fixed breakpoints, checked at 1280, 1024, 768 and 390 px:
- **≤1280:** the dashboard uses two columns, with the services and events column spanning below.
- **≤900:** the sidebar cards become a horizontal tab strip under the header. The dashboard keeps two columns with stretched rows, so panel edges line up. Missions, About and Systems use one column, and the Systems index is no longer sticky.
  - Pitfall: a column switched to `display:flex` without `align-items:stretch` lets panels shrink to their content width, so the panels end up with mismatched widths.
  - The preview's full-page screenshots must unset the internal scroller (`shotfull.mjs`), otherwise everything below the first screen is missing.
- **≤600:** the dashboard switches to one column.
  - The window controls and the last-event clock are hidden. Inline `display:flex` needs `!important` to hide them.
  - The page scrolls normally with `.app` set to `overflow:hidden`, so the grid plane cannot cause horizontal scroll.
  - Form rows stack.
  - Mission-timeline Now/ETA labels go on separate lines, and the session-timeline axis keeps only the start and now labels.
- Preview check: `scratchpad/preview/shotw.mjs <out> <board> <screens> <WxH>` reports `scrollWidth` and the elements that overflow the viewport.

## Approved design (2026-11-15)
The user approved **G1 v2** (`G1v2.dc.html`, plus the `G1v2Tablet` / `G1v2Phone` previews) as the design to implement in `ui/src`. It supersedes "G1 + Motion layer" as the implementation reference. The copy rules, the copy mapping and the responsive breakpoints above all apply.

## Implementation notes (G1 v2 in `ui/src`, 2026-11-15)
- **Plain CSS beside Tailwind.** The design CSS is ported verbatim into `ui/src/g1.css` and imported after `styles.css` in `main.tsx`. The class names follow the design board, so a board tweak maps 1:1 to the app. `biome.json` has an override for this file only (`noImportantStyles`, `noDescendingSpecificity` off), because the responsive rules must beat inline `display:flex`. The SVG ring class was renamed `.ring-p` to avoid a clash.
- **Font.** `@fontsource-variable/saira/wdth.css` (OFL) is bundled. The family name is `"Saira Variable"`, and the width axis (`font-stretch`) gives the condensed headings. No external font requests.
- **View logic lives in `ui/src/lib/*.ts` with vitest tests** (`pnpm test`). `vitest.config.ts` uses the threads pool with `maxThreads: 2` to stay inside the machine's memory limit.
- **Live clock anchor.** Counters such as "Since last kill" and the session time tick from `generated_at + (local now − time the snapshot was received)` (`liveNowMs`, `use-live-now.ts`). The UI therefore does not depend on the browser clock agreeing with the backend clock.
- **Idle rate.** A rate shows `-/h` while its event count is 0. This matches the CLI status line (`src/terminal.rs`). Otherwise the UI formats `value` with one decimal, the same as the backend's `format_rate_per_hour`. Mock fixtures must keep `value` and `display` consistent; the `live_rates` mock used to change only `display`.
- **Thresholds.** The snapshot carries `thresholds.warn_kill_rate_per_hour` and `warn_no_kills_minutes`. `stableSnapshotKey` must include `thresholds`; otherwise a threshold-only change is deduplicated and never re-renders.
- **Focus affordance.** The QA gate requires a focused control to have a non-`none` outline or box-shadow. A border-colour change alone fails, so `.inp:focus-visible` adds a 1 px box-shadow ring.
- **Playwright browsers.** `@playwright/test` 1.61 needs `chromium_headless_shell-1228`. Another tool's playwright-core install can garbage-collect it, which makes every e2e test fail at launch. Restore it with `pnpm --dir ui test:e2e:install`.
- **Shell-scripting pitfall.** `pkill -f "<pattern>"` also matches the invoking shell when the pattern is part of its command line, which kills the shell (exit 144). Keep the background PID (`$!`) and use `kill` on that PID instead.
- **Live-update cues (ported from the design's demo feed onto real snapshots).**
  - A changed value flashes: the KPI kill and scan counts, the mission-kills ring and the header "Last event" clock. `useChangeFlash` returns `""` on mount and then alternates `fx-a` / `fx-b` on each change. The two classes use identical keyframes, so each change restarts the animation.
  - A newly arrived event row glows once (`.ev.fresh::before`, 1 s). `useFreshEventIds` diffs event ids whenever the feed signature changes. The feed present at mount is never marked fresh, and a periodic snapshot with the same feed keeps the same set, so tab switches and polling never replay the cue.
  - Both cues live in the motion layer, so `.m-off` and `prefers-reduced-motion` disable them.
  - Mock `?mock_state=live_feed` adds one kill event on the second load, and the e2e test `@live-updates` covers it.
- **Mock fixtures must stay internally consistent.** The `empty` scenario now clears `last_kill_*` / `last_scan_*`; before this it showed a since-last-kill timer with zero kills.
