# Header logo presentation

- The header brand mark in `ui/src/components/dashboard/dashboard-shell.tsx` should render as the standalone `/logo.png` image.
- Do not add a square frame around the logo: avoid border, background, padding, rounded frame, or inset shadow on the logo image.
- Keep the standalone logo vertically aligned with the `ED-SENTRY` label structurally, not with a magic translate. The logo image stays untransformed at `size-6`, while the adjacent wordmark uses `leading-none` so its 14px text line participates cleanly in the parent `items-center` row.
- Browser QA evidence for the standalone logo should show `borderTopWidth=0px`, transparent background, `paddingTop=0px`, and `boxShadow=none` at desktop and mobile widths.
