# Frontend Reference Style Alignment - 2026-06-22

User asked to make the implemented WebUI style match `reference-design/design1.tsx` as closely as possible without modifying the reference file.

Key alignment decisions:
- `reference-design/design1.tsx` remains untouched.
- Desktop shell now follows the reference top HUD: one-line `h-14` header, orange active top tabs, `ed-sentry` brand at left, status at right.
- Mobile keeps the same visual language but hides the right status text below `sm` so full tab labels do not clip at 375px. This preserves no-horizontal-overflow QA while staying close to the reference style.
- Tactical primitives were tightened to the reference colors and effects: `#03060a` grid background, `#060a11` HUD panels, orange title strips, orange corner marks, slate insets, and orange/emerald/amber/rose/cyan status colors.
- Telemetry now uses the reference `lg:grid-cols-4` layout, fixed 320px summary panels, bounded Recent Alerts, and reference-like mission/event row treatments.
- Missions now uses the reference master/detail shape: `flex`, left `w-1/3` mission directory, right `flex-1` mission intel, orange selected rows, and tracking uplink detail box.
- Events feed now uses the reference panel-internal sticky header (`-top-4`, `-mx-4`, dark blurred header) instead of the previous zero-padding table variant.
- Systems/config now uses a centered `max-w-4xl` tactical panel with floating orange section labels. The core reference sections are preserved in spirit (`Local ingestion`, `Matrix relay protocol`, `Local API gateway`), and extra editable Monitor/Log sections reuse the same section treatment to preserve the user's config-editing requirement.
- Matrix token remains write-only: UI displays only `TOKEN PRESENT IN VAULT` / `NO TOKEN IN VAULT`; the stored token is never rendered.

Verification after alignment:
- `pnpm --dir ui lint` passed.
- `pnpm --dir ui typecheck` passed.
- `pnpm --dir ui test:e2e -- --project=chromium` passed: 25/25.
- Final normal `pnpm --dir ui build` passed after e2e mock build, restoring the default production web adapter build.
- Responsive evidence screenshots updated under `.omo/evidence/gui-webui-tauri/`, including `task-12-desktop.png`, `task-12-tablet.png`, and `task-12-mobile.png`.

Known tradeoff:
- Exact desktop reference dimensions are preserved more strictly than mobile. On mobile, the right connection status is hidden to prevent tab clipping and horizontal overflow; this is an intentional responsive adaptation rather than a desktop-style deviation.
