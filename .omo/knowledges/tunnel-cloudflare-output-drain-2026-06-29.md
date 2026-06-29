## Cloudflare Quick Tunnel output drain fix

When `cloudflared tunnel --url ...` creates a Quick Tunnel, it emits the public
`trycloudflare.com` URL and then continues printing connectivity precheck,
metrics, and connection logs. Provider output readers must keep draining stdout
and stderr after the first URL is detected. If the reader returns immediately,
the child process can block or terminate once the OS pipe fills, which makes the
UI show the URL briefly and then lose the running tunnel status.

The provider now sends the first parsed URL reliably with `send().await`, marks
the URL as reported, and continues reading lines until EOF or provider cleanup
aborts the task. Regression coverage is
`tunnel_provider_drains_output_after_url_is_reported` in `tests/tunnel_provider.rs`.

The provider also starts `cloudflared tunnel` with `--metrics 127.0.0.1:0`.
This asks `cloudflared` to bind its metrics listener to an OS-selected loopback
port instead of competing for the default metrics port/range. It avoids Windows
runtime failures where a manual or previous `cloudflared` process leaves the
tunnel URL visible briefly before startup collapses around metrics binding.

If the public URL remains reachable but disappears from the UI, the likely
failure path is snapshot overwrite, not provider exit. Manual tunnel start
publishes a checked `Running` tunnel snapshot, while unrelated monitor runtime
updates can publish snapshots whose tunnel field is still the unchecked startup
default. Event snapshot merging must preserve the latest checked tunnel status
when an incoming runtime snapshot has no checked tunnel timestamp.

Manual `cloudflared.exe tunnel --url http://127.0.0.1:<port>` URLs are not
trusted by the app's Host policy, because the backend only trusts the active
tunnel host/session registered by the in-app provider. A manually started Quick
Tunnel can therefore reach the backend but receive `host_rejected`; that is the
intended protection, not a DNS failure once the domain resolves.
