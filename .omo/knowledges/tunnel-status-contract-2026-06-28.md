# Tunnel Status Contract

Task 2 added `src/app/tunnel.rs` as the narrow app-layer tunnel status contract. It intentionally models only `cloudflare_quick` and inert future `ssh`; it does not add provider discovery, a registry, plugin loading, or SSH execution.

Snapshot shape now has a separate `tunnel` object. Keep public tunnel URLs in `snapshot.tunnel.public_url`; do not reuse or overload `snapshot.web.url`, which remains the local WebUI URL.

The serialized tunnel status values are exactly `disabled`, `start`, `starting`, `running`, `error`, and `unsupported`. `TunnelProvider` serde accepts only `cloudflare_quick` and `ssh`; unknown providers must fail parsing at the serde boundary.

Normal verification for this surface is:

```bash
cargo test --all tunnel_status
cargo test --test tunnel_status tunnel_status_manual_qa_serializes_running_cloudflare_fixture -- --ignored --nocapture
```

Follow-up fix added the narrow lifecycle contract in `src/app/tunnel/provider.rs`: `TunnelProviderController`, `TunnelManager<P>`, `CloudflareQuickTunnelProvider`, and inert `SshTunnelProvider`. This is intentionally not a registry or plugin system. Cloudflare start/stop/status are data-only placeholders until child-process work; SSH start/stop/status all return `unsupported`.
