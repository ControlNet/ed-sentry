# LAN WebUI policy

The WebUI is intended to work for LAN clients when `web.host` is `0.0.0.0` or another non-loopback bind host.

- Keep Host validation: requests must still match the configured bind policy.
- Keep the CORS allow-origin list as-is for browser cross-origin fetch behavior.
- Do not require localhost Origin for `PUT /api/config`.
- Do not require localhost Origin for WebSocket `/api/events` or `/ws`.
- Do not reject non-loopback WebSocket clients solely because the peer address is remote.

Manual QA signal: with the service bound to `0.0.0.0`, a LAN-style `Host`/`Origin` config `PUT` should return `200 OK`, and a WebSocket upgrade through a non-loopback local IP should return `101 Switching Protocols` without `origin_rejected` or `remote_websocket_rejected`.
