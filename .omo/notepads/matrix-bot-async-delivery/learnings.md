## Task 3 - Dependency and async runtime foundation

- `src/main.rs` can use `#[tokio::main] async fn main() -> ExitCode` while preserving the existing single error-printing path by converting `build_runtime_command(cli)` into an async `run_command(command).await` result before the final `match`.
- `run_watch` remains behavior-equivalent with terminal-only delivery when only `thread::sleep(live_poll_interval(config))` is replaced by `tokio::time::sleep(live_poll_interval(config)).await`.
- `run_replay` can be async-compatible without adding awaits or Matrix construction; replay remains terminal-only and continues to use the existing synchronous journal stream path.

## 2026-06-15 - Task 2 notification routing semantics

- `Notification::new` is the single mention gate; routing now treats level `0` as no delivery, level `1` as notify without mention, and level `2+` as notify with Matrix mention.
- Existing critical default log levels that were `3` were lowered to `2` so they remain mention-capable under the new public contract without changing notification text or event types.

## 2026-06-15 - Task 1 Matrix config model

- `AppConfig::load_optional(None)` now treats root `config.toml` as an optional implicit config file; an explicit `--config` path still calls strict `load_from_path`.
- `[matrix] enabled = true` is parsed into `MatrixConfig` with optional required-runtime fields preserved for Task 6 validation, while missing `[matrix]` and `enabled = false` both produce `None` silently.
- `MatrixConfig` redacts `access_token` in `Debug`, and `/config.toml` is ignored so local token-bearing config stays out of git by default.

## 2026-06-15 - Task 4 documentation and example config contract

- The public config contract documents only `config.example.toml` and local `config.toml`; no alternate local config path is described.
- The Matrix MVP docs use direct `access_token = "<token>"` in local `config.toml` and explicitly tell users not to commit that file.
- Replay is documented as terminal-only, while Matrix delivery is watch-mode only and requires an unencrypted room because E2EE is unsupported.

## 2026-06-15 - Task 5 EventMonitor notification producer

- `EventMonitor` is no longer generic over a notifier and owns no `NotificationDispatcher`; it stores only monitor state/config and returns `Notification` models from `process_event`, `check_warnings_at`, `start_monitor`, and `finish`.
- Watch and replay now keep `NotificationDispatcher<TerminalNotifier<_>>` in `src/main.rs`, so terminal delivery remains caller-owned while monitor business logic stays synchronous.
- Level `0` monitor outputs are now visible as returned `Notification` models for caller routing; `NotificationDispatcher` remains the delivery boundary that suppresses level `0` sends.

## 2026-06-15 - Task 7 async terminal delivery and fanout layer

- `DeliveryHub<W>` is the new caller-owned async delivery boundary; it keeps terminal output on `TerminalNotifier<W>`, filters level `0` notifications before any sink, and returns remote failures as `DeliveryWarning` values.
- `RemoteDelivery` is intentionally only an async trait plus optional boxed sink in `DeliveryHub`; watch and replay currently construct terminal-only hubs, leaving Matrix SDK construction and actual sends to later tasks.
- Replay keeps journal parsing and `EventMonitor` synchronous by collecting producer notifications during the sync stream pass, then awaiting terminal-only delivery after the stream completes.

## 2026-06-15 - Task 6 Matrix runtime config validation

- `MatrixConfig::to_runtime_config()` now keeps parse-time Matrix fields optional but converts enabled complete config into `MatrixRuntimeConfig` only when `homeserver`, `user_id`, `room_id`, and `access_token` are present.
- Missing required Matrix runtime fields produce exactly one line-safe warning listing field names only and return no runtime Matrix config for that run; missing or disabled Matrix stays silent.
- `MatrixRuntimeConfig::device_id()` returns the fixed Matrix device ID `EDAFKDASHBOARD`, and debug output for parse/runtime/result/app/runtime wrappers remains access-token redacted.

## 2026-06-15 - Task 8 async watch shutdown and status cadence plumbing

- `StatusCadence` lives in `src/delivery.rs` and defaults to 60 seconds; watch mode builds it from the configured Matrix `status_update_interval_seconds` when present, without changing `src/config.rs`.
- `DeliveryHub::publish_status()` still renders terminal status on every call, but optional remote status publication is cadence-gated inside the hub and can be forced for explicit seams.
- Watch mode now forces the startup status publish seam and uses normal cadence checks for later poll-loop status updates; malformed Journal/config warnings remain direct stderr writes rather than notifications.

## 2026-06-15 - Task 9 Matrix SDK delivery implementation

- `matrix-sdk` 0.18.0 supports the required startup path with `Client::builder().homeserver_url(...).build().await`, `MatrixAuth::restore_session(MatrixSession, RoomLoadSettings::default()).await`, `Client::sync_once(SyncSettings::default()).await`, and `Client::get_room(&OwnedRoomId)`.
- The access-token restore session uses `MatrixSession { meta: SessionMeta { user_id, device_id }, tokens: SessionTokens { access_token, refresh_token: None } }`; the fixed Task 6 device ID is passed through `MatrixRuntimeConfig::device_id()` rather than a user-configurable value.
- Runtime status edits use `Room::make_edit_event(original_event_id, EditedContent::RoomMessage(RoomMessageEventContentWithoutRelation::text_plain(...)))` followed by `Room::send(...)`; the delivery layer keeps the original status event ID and intentionally does not replace it with later edit event IDs.
- Fakeable Matrix sender tests can validate plain body text, structured `m.mentions`, timeout/error redaction behavior, and original-event status edit semantics without real homeserver credentials or network access.

## 2026-06-15 - Task 10 watch-mode Matrix integration

- Watch mode now constructs a caller-owned `DeliveryHub` before replaying preload records, validates runtime Matrix config only in `run_watch`, and attaches `MatrixDelivery::connect(...).await` only after a successful connection.
- Replay remains Matrix-free by construction: it never calls `RuntimeConfig::matrix_runtime()`, never connects Matrix, and continues to build a terminal-only hub even when implicit `config.toml` enables Matrix.
- The debug-only fake Matrix sink is process-testable through `ED_AFK_DASHBOARD_FAKE_MATRIX_*` environment variables and records connect/send/status JSONL without contacting a homeserver.

## 2026-06-15 - Task 11 README and knowledge update

- README now documents the final watch-mode Matrix contract with copy-pastable config, watch, explicit-config, replay, and verification commands.
- Replay is documented as isolated from Matrix initialization, sends, and status publication even when local `config.toml` contains Matrix settings.
- `.omo/knowledges/matrix-bot-async-delivery.md` records the final async main, producer monitor, `DeliveryHub`, Matrix SDK access-token restore, no-E2EE, replay-isolation, gitignored config, and level-semantics contract.

## 2026-06-15 - Task 12 release-readiness QA

- Full release-readiness validation passed without source/test fixes: `cargo fmt --check`, `cargo test --all`, `cargo clippy --all-targets --all-features -- -D warnings`, help CLI, replay error CLI, replay fixture CLI, `git check-ignore config.toml`, and staged secret scan all satisfied the Task 12 acceptance criteria.
- Matrix message formatting remains routed through `Notification.remote_text`: `notification_content` combines optional mention, optional emoji, and sanitized remote text, while raw Journal payload access stays outside `src/matrix.rs`.
- Replay QA confirmed `--replay --no-status-line` fails with `Error: replay requires --set-file <file>` and the sanitized combat fixture exits `0` with terminal-only replay output.
