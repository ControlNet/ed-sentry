# Matrix Bot Async Delivery

- `src/main.rs` now runs through an async Tokio main path. Watch mode awaits delivery work while replay keeps its deterministic stream pass isolated from remote delivery.
- `EventMonitor` is a producer. It owns monitor state and returns `Notification` models, while callers decide where those notifications go.
- `DeliveryHub` is the async delivery boundary. It always keeps terminal delivery, filters level `0`, and may attach Matrix only for watch mode.
- Matrix delivery uses `matrix-sdk` access-token session restore with the fixed device ID `EDAFKDASHBOARD`, then sends plain text from `Notification.remote_text` with structured mentions when configured.
- No E2EE. Matrix end-to-end encryption is not supported. Users must choose an unencrypted Matrix room.
- Replay isolation is intentional. Replay never validates Matrix config, connects Matrix, sends Matrix messages, or publishes Matrix status, even when local Matrix config exists.
- `config.example.toml` is the safe committed template. Root `config.toml` is local, gitignored, and must not be committed because it can hold `access_token = "<token>"`.
- Notification levels are public config semantics: `0 = off`, `1 = notify`, and `2+ = notify + Matrix mention when Matrix delivery and mention config are present`.
- Built-in default `2` mention-level categories are intentionally limited to `fighter_down`, `died`, `cargo_lost`, `fuel_low`, `fuel_critical`, `missions_all`, `rank_promotion`, and `no_kills`. Other default notifications, including easy/hard kills, scans, navigation/session lifecycle, summaries, low kill rate, fighter launch, and shutdown, stay level `1` unless the user raises them in config.
- `rank_promotion` covers the Elite Journal `Promotion` event plus raw `PowerplayRank` and `SquadronPromotion` events. These default to Matrix mention level because the user wants rank-up related notifications to be prominent.
- `matrix-sdk` must stay on `default-features = false` for this app. The default Matrix SDK feature bundle enables E2EE and SQLite storage, which are out of scope and pulled `libsqlite3-sys` into Windows GNU cross-builds. Access-token session restore, sync, room lookup, send, edit, and structured mention APIs compile and pass tests without the default feature bundle.
- Windows release zip layout is `ed-sentry/ed-sentry.exe` plus `ed-sentry/config.toml`. The packaged config is copied from `config.example.toml` as a safe editable template; never package a local real `config.toml` with tokens.
- Matrix `room_id` accepts either a room ID (`!room:server`) or a legal room alias (`#alias:server`). Alias strings using `@` as the server separator are invalid and should fail validation.
- Watch mode keeps CLI preload behavior, but Matrix only receives a startup header plus notifications whose Journal event timestamp is at or after the program start time. Historical preload notifications remain terminal-only.
