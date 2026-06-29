Tunnel visual QA follow-up: after background visual QA, keep post-auth config feedback distinct from loading state. In `ConfigPanel`, retrying after tunnel auth should render the loading section while `loadConfig()` is pending, then show either no stale loading text or a clear success message such as `Tunnel access unlocked` once the config form is ready.

The QR portal must be viewport-safe. `TunnelLinkQr` uses a fixed 128px visual box (`size-28` image plus `p-2`) and should clamp its left/top coordinates to an 8px viewport margin, flipping above the anchor when below-anchor placement would exceed the viewport bottom.

For the tactical dark panel, `FieldMessage` info text should use at least `text-slate-400`; `text-slate-500` was too muted for normal-size info copy.
