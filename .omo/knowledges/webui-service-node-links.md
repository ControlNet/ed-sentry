# WebUI service node links

Service Nodes render the Web Interface bind address as the visible detail text. When `snapshot.web.message` is absent and `snapshot.web.url` is present, that detail should be an accessible anchor using `snapshot.web.url` as `href`.

The visible text intentionally remains the bind address. When the bind host is `0.0.0.0`, the link target uses `localhost` instead, e.g. visible `0.0.0.0:8765` opens `http://localhost:8765`. Non-wildcard hosts remain unchanged, e.g. visible `192.168.50.10:8765` opens `http://192.168.50.10:8765`.

Coverage: `ui/e2e/dashboard-smoke.spec.ts` has a mock-state assertion for the clickable Web Interface URL.
