# ed-sentry Design System

## 1. Atmosphere & Identity

ed-sentry is a dark operational dashboard for long-running Elite Dangerous AFK monitoring. It should feel like a local command console: dense, low-glare, technical, and status-first, with the current watch state always more important than decoration. The signature is "orbital telemetry": quiet dark surfaces, thin instrument borders, compact mono data, and deliberate green/amber/red status signals that make health, risk, Journal input, Matrix delivery, and mission progress scannable at a glance. This is not a marketing hero, not a landing page, and not a generic admin-template skin.

## 2. Color

### Palette Tokens

All UI colors must resolve through these tokens. Hex values are allowed only in this palette table; component prose and implementation code must reference tokens.

| Role | Token | Dark Value | Usage |
| --- | --- | --- | --- |
| App background | `--color-bg-base` | #03070B | Root viewport and outer shell |
| App background inset | `--color-bg-inset` | #070D13 | Navigation rail, table gutters, empty zones |
| Panel surface | `--color-surface-panel` | #0B1218 | Dashboard panels and form sections |
| Panel raised | `--color-surface-raised` | #101A22 | Metric tiles, selected rows, popovers |
| Surface pressed | `--color-surface-pressed` | #16232D | Active navigation and pressed controls |
| Border subtle | `--color-border-subtle` | #17242E | Low-emphasis dividers and table row separators |
| Border strong | `--color-border-strong` | #28404D | Panel frames, table boundaries, active outlines |
| Text primary | `--color-text-primary` | #E6F0F4 | Main labels, values, headings |
| Text secondary | `--color-text-secondary` | #A7B8C0 | Supporting copy, timestamps, metadata |
| Text muted | `--color-text-muted` | #667A84 | Disabled, placeholder, inactive feed text |
| Accent action | `--color-accent-action` | #4BB7C8 | Links, focus rings, selected shell item |
| Accent action hover | `--color-accent-action-hover` | #7AD7E3 | Hover and active action state |
| Tactical accent | `--color-tactical-accent` | #F97316 | Reference-design tactical tabs, HUD panel accents, mission progress emphasis |
| Status online | `--color-status-online` | #5ED38C | Healthy watch, connected Matrix, active Journal |
| Status warning | `--color-status-warning` | #E8B75D | Idle risk, low rate, degraded delivery |
| Status danger | `--color-status-danger` | #F06464 | Death, critical fuel, disconnected required service |
| Status info | `--color-status-info` | #7AA7FF | Neutral lifecycle events and configuration info |
| Status neutral | `--color-status-neutral` | #7A8E98 | Unknown, paused, unavailable, not configured |
| Data scan | `--color-data-scan` | #69D6D0 | Cargo scan counts and scan events |
| Data kill | `--color-data-kill` | #FF8A6A | Kill counts and combat feed emphasis |
| Data mission | `--color-data-mission` | #B9A7FF | Mission progress, merits, faction totals |
| Overlay scrim | `--color-overlay-scrim` | #010305 | Dialog and drawer backdrop |
| Focus ring | `--color-focus-ring` | #7AD7E3 | Keyboard focus indicator |

### shadcn/ui Token Mapping

shadcn/ui is component source scaffolding only. It must be rethemed through this contract before any component ships.

| shadcn Token | ed-sentry Token |
| --- | --- |
| `--background` | `--color-bg-base` |
| `--foreground` | `--color-text-primary` |
| `--card` | `--color-surface-panel` |
| `--card-foreground` | `--color-text-primary` |
| `--popover` | `--color-surface-raised` |
| `--popover-foreground` | `--color-text-primary` |
| `--primary` | `--color-accent-action` |
| `--primary-foreground` | `--color-bg-base` |
| `--secondary` | `--color-surface-raised` |
| `--secondary-foreground` | `--color-text-secondary` |
| `--muted` | `--color-bg-inset` |
| `--muted-foreground` | `--color-text-muted` |
| `--accent` | `--color-surface-pressed` |
| `--accent-foreground` | `--color-text-primary` |
| `--destructive` | `--color-status-danger` |
| `--border` | `--color-border-subtle` |
| `--input` | `--color-border-strong` |
| `--ring` | `--color-focus-ring` |

### Color Rules

- No raw token-free colors in React, CSS, Tailwind utilities, tests, screenshots, or docs.
- Status colors are semantic, not decorative. Green means healthy or complete, amber means degraded or attention needed, red means critical or failed.
- Data colors are reserved for metrics and event categories: scan, kill, and mission.
- Text contrast must remain readable on `--color-bg-base`, `--color-surface-panel`, and `--color-surface-raised`.
- Avoid purple-blue gradient dashboards, soft decorative blobs, glass cards, and stock SaaS styling.

## 3. Typography

### Font Stack

- Primary UI: `Fira Sans`, `Inter`, `system-ui`, `-apple-system`, `BlinkMacSystemFont`, `Segoe UI`, sans-serif.
- Mono/data: `Fira Code`, `JetBrains Mono`, `SFMono-Regular`, `Consolas`, monospace.
- No serif family in the application shell.

### Type Tokens

| Token | Size | Weight | Line Height | Tracking | Usage |
| --- | --- | --- | --- | --- | --- |
| `--font-title` | 24px | 650 | 32px | 0 | App title, primary view title |
| `--font-section` | 18px | 650 | 26px | 0 | Panel headings and table titles |
| `--font-subsection` | 15px | 600 | 22px | 0 | Tile labels, grouped form titles |
| `--font-body` | 14px | 400 | 22px | 0 | Default body, forms, table cells |
| `--font-body-strong` | 14px | 600 | 22px | 0 | Active labels and emphasized cells |
| `--font-caption` | 12px | 500 | 16px | 0 | Metadata, hints, feed timestamps |
| `--font-overline` | 11px | 700 | 14px | 0.06em | Uppercase panel labels only |
| `--font-metric` | 28px | 650 | 34px | 0 | Primary Metric Tile values |
| `--font-metric-sm` | 20px | 650 | 26px | 0 | Compact Metric Tile values |
| `--font-code` | 13px | 500 | 20px | 0 | Journal paths, Matrix IDs, config keys |
| `--font-tactical-overline` | 10px | 700 | 14px | 0.14em | Tactical HUD labels, tab labels, compact badges |
| `--font-tactical-micro` | 9px | 600 | 13px | 0.16em | Dense telemetry metadata and mission IDs |

### Typography Rules

- Use mono/data type only for values users compare or copy: Journal path, timestamps, rates, credits, mission IDs, config keys, Matrix room IDs.
- Do not scale font size with viewport width. Use token swaps at breakpoints instead.
- Body text must never drop below `--font-caption`.
- Long commander, system, faction, path, and Matrix values must truncate or wrap by component rule, never overflow.
- Headline-scale type is reserved for operational view titles, not panels inside the dashboard.

## 4. Spacing & Layout

### Base Unit

All spacing derives from a 4px base. Layout and component code must use these tokens rather than magic spacing values.

| Token | Value | Usage |
| --- | --- | --- |
| `--space-0` | 0 | Flush joins and reset state |
| `--space-1` | 4px | Icon gaps, tight metadata joins |
| `--space-2` | 8px | Inline control gaps, compact row padding |
| `--space-3` | 12px | Default row gap, field padding |
| `--space-4` | 16px | Tile padding, shell item padding |
| `--space-5` | 20px | Panel compact padding |
| `--space-6` | 24px | Panel comfortable padding, page gutter |
| `--space-8` | 32px | Dashboard group gap |
| `--space-10` | 40px | Major vertical separation |
| `--space-12` | 48px | Empty-state internal spacing |

### Shape And Layout Tokens

| Token | Value | Usage |
| --- | --- | --- |
| `--radius-xs` | 2px | Status dots, progress fills |
| `--radius-sm` | 4px | Inputs, badges, table row focus |
| `--radius-md` | 6px | Panels, tiles, popovers |
| `--radius-lg` | 8px | Dialogs and major drawers only |
| `--shell-nav-width` | 232px | Desktop navigation rail |
| `--shell-topbar-height` | 56px | Compact desktop top bar |
| `--content-max-width` | 1440px | Dashboard content cap |
| `--metric-min-width` | 180px | Metric Tile responsive minimum |
| `--feed-row-min-height` | 44px | Event Feed row stability |
| `--table-row-height` | 48px | Mission Table row stability |
| `--tactical-summary-panel-height` | 320px | Telemetry mission/feed summary panel height |
| `--tactical-workspace-min-height` | 544px | Mission and event workspace minimum height |

### Layout Rules

- The first viewport is the live dashboard: shell navigation, Status summary, Metric grid, Event Feed, Mission Table, Journal source, Matrix Status, and config entry points.
- Desktop uses a persistent left shell navigation and a constrained dashboard canvas. Mobile collapses navigation into a top bar plus drawer.
- Use CSS grid for the main dashboard. Preferred desktop grid: 12 columns with `--space-6` gutters.
- Panels are not nested inside cards. Use section bands and dashboard panels, not card stacks inside card stacks.
- Fixed-format elements such as Metric Tiles, Status badges, table rows, and icon buttons must have stable dimensions to prevent live-update layout shift.

## 5. Components

### Shell Navigation

- **Structure**: app title, current runtime badge, primary links, compact footer status, no marketing content.
- **Variants**: desktop rail, mobile drawer, Tauri window-aware top bar.
- **Spacing**: `--space-4` item padding, `--space-2` icon-to-label gap, `--space-6` rail padding.
- **States**: default, hover, active, keyboard focus, disabled when a feature is unavailable.
- **Rules**: use Lucide or Radix-compatible SVG icons only. No emoji icons. Active state uses `--color-surface-pressed`, `--color-accent-action`, and `--color-border-strong`.

### Dashboard Panel

- **Structure**: heading row, optional right-side controls, body region, optional footer metadata.
- **Variants**: default, dense, warning, danger, loading, empty.
- **Spacing**: `--space-5` padding by default; `--space-4` in dense panels.
- **States**: loading skeleton, empty, error, degraded, keyboard-focused control within panel.
- **Rules**: panels use `--color-surface-panel`, `--color-border-subtle`, `--radius-md`, and tokenized depth from Section 7.

### Metric Tile

- **Structure**: short label, mono value, optional delta/rate, optional spark-free status strip.
- **Variants**: scan, kill, bounty, merits, shield, hull, fighter, fuel, session time.
- **Spacing**: `--space-4` padding, `--space-2` vertical label/value gap.
- **States**: neutral, improving, warning, danger, stale, loading.
- **Rules**: use `--font-metric` or `--font-metric-sm`. Use `--color-data-scan`, `--color-data-kill`, or `--color-data-mission` only when the data role matches.

### Event Feed

- **Structure**: timestamp, severity marker, event category, message, optional source badge.
- **Variants**: lifecycle, scan, kill, warning, Matrix delivery, Journal source, config change.
- **Spacing**: each row min height uses `--feed-row-min-height`; row padding uses `--space-3`.
- **States**: live, buffered, unread, selected, copied, empty, disconnected.
- **Rules**: never show raw Journal lines or private commander/chat content. Event text must be line-safe and frontend-safe.

### Mission Table

- **Structure**: mission name/type, faction, target/current progress, reward or merits, state, updated timestamp.
- **Variants**: active, complete, failed, stale, unknown.
- **Spacing**: row height uses `--table-row-height`; cell gap uses `--space-3`.
- **States**: sorted, filtered, selected, empty, loading, stale.
- **Rules**: progress bars use `--color-data-mission`; status cells use semantic Status Badge variants.

### Status Badge

- **Structure**: optional SVG icon, label, optional mono value.
- **Variants**: online, warning, danger, info, neutral, paused, disabled.
- **Spacing**: `--space-2` horizontal padding, `--space-1` icon gap.
- **States**: default, hover when clickable, focus, disabled.
- **Rules**: badge color must map to a status token. Do not invent one-off severity colors.

### Config Form

- **Structure**: grouped fieldsets for Journal, monitor warnings, Matrix, and WebUI settings.
- **Variants**: read-only, editable, dirty, saving, saved, validation-error, permission-error.
- **Spacing**: field gap uses `--space-4`; group gap uses `--space-6`.
- **States**: focus, invalid, disabled, saving, success, error.
- **Rules**: Matrix access tokens are write-only. Show token-present state without echoing raw token values. Config keys use `--font-code`.

### Journal Source Panel

- **Structure**: current file/folder, last offset or timestamp, tailing state, file-select action.
- **Variants**: active, missing, permission-error, stale, replay-unsupported.
- **Rules**: paths use `--font-code`, truncate in the middle when needed, and never leak private Journal content.

### Matrix Status Panel

- **Structure**: enabled state, homeserver, room identifier, delivery state, last delivery result, next status cadence.
- **Variants**: disabled, configured, connected, degraded, failed.
- **Rules**: use `--color-status-neutral` when Matrix is intentionally disabled. Never render raw access tokens.

## 6. Motion & Interaction

### Motion Tokens

| Token | Duration | Easing | Usage |
| --- | --- | --- | --- |
| `--motion-instant` | 80ms | ease-out | Press feedback and quick hover |
| `--motion-fast` | 140ms | ease-out | Button, badge, table row hover |
| `--motion-standard` | 220ms | ease-in-out | Drawer, popover, tab, panel state |
| `--motion-emphasis` | 360ms | cubic-bezier(0.16, 1, 0.3, 1) | Initial dashboard hydration only |

### Interaction Rules

- Animate only `transform`, `opacity`, and color transitions. Never animate layout dimensions.
- Real-time updates should feel calm: prefer subtle row insertion, status pulse opacity, or timestamp change over bouncing motion.
- Every interactive element must have hover, active, focus-visible, disabled, loading, and error affordances when applicable.
- Respect `prefers-reduced-motion` by removing non-essential transitions and pulses.
- Focus rings use `--color-focus-ring` and must remain visible on every surface token.

## 7. Depth & Surface

### Strategy

Depth strategy is tonal-shift with tokenized hairline separation. Avoid heavy shadows, glassmorphism, blur layers, and decorative glows. The dashboard should read as instrument layers in a dark cockpit, not floating SaaS cards.

| Depth Token | Value | Usage |
| --- | --- | --- |
| `--depth-base` | `background: var(--color-bg-base)` | App root |
| `--depth-inset` | `background: var(--color-bg-inset)` | Rail, table header, recessed regions |
| `--depth-panel` | `background: var(--color-surface-panel); border: 1px solid var(--color-border-subtle)` | Standard panels |
| `--depth-raised` | `background: var(--color-surface-raised); border: 1px solid var(--color-border-strong)` | Metric Tile, active row, popover |
| `--depth-pressed` | `background: var(--color-surface-pressed); border: 1px solid var(--color-border-strong)` | Active shell item and pressed control |
| `--depth-overlay` | `background: var(--color-overlay-scrim)` | Dialog scrim |

### Surface Rules

- Use elevation to clarify state and hierarchy, not to decorate.
- A panel may use either `--depth-panel` or `--depth-raised`; nested raised surfaces must be functional controls, table selections, or popovers.
- Borders must come from `--color-border-subtle` or `--color-border-strong`.
- shadcn/ui defaults must be audited against these surface rules before use.
