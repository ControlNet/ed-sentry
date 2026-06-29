# Tunnel UI and configuration requirements - 2026-06-28

User requirements captured for the planned zero-config tunnel feature:

- Tunnel startup must be configurable from TOML with an `auto_start` setting.
  - If `auto_start = true`, tunnel starts automatically when WebUI is enabled/started.
  - If `auto_start = false`, tunnel does not start initially and waits for manual user action.
- GUI should add a new `TUNNEL` entry under `TELEMETRY, SERVICE NODES`.
- The right-side service status control for `TUNNEL` should show states like `START` and `RUNNING`.
  - `START` means the tunnel is available to start and should be clickable.
  - Clicking `START` starts the tunnel and changes the state to `RUNNING` after startup succeeds.
  - `RUNNING` should expose/display the concrete public tunnel link.
- Tunnel link behavior:
  - Hovering over the link should render a QR Code for convenient phone access.
  - Clicking the link should directly open the public tunnel URL.
- Security/product behavior for tunnel access:
  - Public tunnel access is higher risk and must not allow unrestricted access to the `SYSTEMS` page.
  - When accessed through the tunnel, the `SYSTEMS` page may still be clickable, but should render a “not accessible under tunnel / password authentication required” style notice instead of the normal page.
  - The password is configured in TOML as plain text for this stage; local-at-rest password security is intentionally out of scope for the first implementation.
  - Tunnel config must include a `config_password` value containing the plain-text password.
  - If `config_password` is empty/missing, tunnel startup is still allowed and tunnel visitors may access `SYSTEMS` content and configuration without password authentication; the user accepts this risk.
  - Tunnel authentication should use a JWT-like/session-token approach implemented with a popular, maintained library rather than a hand-rolled token format.
  - When `config_password` is configured, both the `SYSTEMS` page and related backend APIs must require successful tunnel authentication under tunnel access.
  - After successful tunnel authentication, remote tunnel users are allowed to modify configuration; do not make tunnel access read-only.
  - Tunnel startup failures must not prevent the WebUI/app from running. Show an error/warning state and allow retry.
  - Do not show Cloudflare Quick Tunnel “best effort / no SLA / dev only” disclaimers in the UI.
- Config layout decision:
  - Use a top-level `[tunnel]` TOML table, not `[web.tunnel]`.

Planning implication:

- Keep tunnel startup attached to the shared WebUI runtime path so CLI watch and GUI both use the same behavior, but add GUI manual-start control and tunnel status rendering.
- Extend Web/Tunnel status DTOs so the UI can distinguish `START`/not running, starting, running, warning/error, and expose the public URL only when available.
- Add tunnel-origin/access-mode detection so protected UI routes such as `SYSTEMS` can render the tunnel restriction/auth prompt.
- Auth/access-mode policy must distinguish local access from tunnel access: local WebUI/desktop should keep existing behavior; tunnel access applies password/session checks only when `config_password` is non-empty.

Strict UI scope constraint:

- Do not modify unrelated UI content, layout, styling, or behavior.
- Allowed UI changes are limited to:
  1. The tunnel-mode login/auth prompt that appears when accessing the `SYSTEMS` page through tunnel access and `config_password` is configured.
  2. The new `TUNNEL` entry/status/link/QR behavior under `TELEMETRY, SERVICE NODES`.
  3. The `SYSTEMS` page/config UI additions needed to align with the new `[tunnel]` configuration fields.
- No broader redesign, polish pass, text changes, layout changes, or component rewrites outside these areas unless explicitly requested later.
