use chrono::{TimeZone, Utc};
use ed_sentry::app::{
    AppSnapshot, CloudflareQuickTunnelProvider, MatrixStartupStatus, SshTunnelProvider,
    TunnelManager, TunnelProvider, TunnelSession, TunnelSessionId, TunnelStatus, TunnelStatusKind,
    WebStartupStatus,
};
use ed_sentry::mission::MissionTracker;
use ed_sentry::state::SessionState;
use serde_json::json;

fn fixture_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 6, 28, 12, 0, 0)
        .single()
        .unwrap()
}

#[test]
fn tunnel_status_characterizes_current_web_and_matrix_snapshot_serialization() {
    // Given: existing Matrix and Web startup statuses with distinct URLs/room fields.
    let now = fixture_time();
    let snapshot = AppSnapshot::from_state(
        &SessionState::new(),
        &MissionTracker::new(),
        now,
        MatrixStartupStatus::running("!fixture:matrix.invalid", now),
        WebStartupStatus::running("127.0.0.1:8765", "http://127.0.0.1:8765", now),
    );

    // When: the snapshot is serialized for the API/UI boundary.
    let value = serde_json::to_value(&snapshot).unwrap();

    // Then: Web and Matrix keep their current independent status shape.
    assert_eq!(value["matrix"]["kind"], "running");
    assert_eq!(value["matrix"]["status_label"], "Running");
    assert_eq!(value["matrix"]["room_id"], "!fixture:matrix.invalid");
    assert_eq!(value["web"]["kind"], "running");
    assert_eq!(value["web"]["status_label"], "Running");
    assert_eq!(value["web"]["bind_address"], "127.0.0.1:8765");
    assert_eq!(value["web"]["url"], "http://127.0.0.1:8765");
    assert!(value["web"].get("public_url").is_none());
}

#[test]
fn tunnel_status_serializes_disabled_and_manual_start_states() {
    // Given: Cloudflare Quick tunnel statuses before a child process exists.
    let disabled = TunnelStatus::disabled(TunnelProvider::CloudflareQuick);
    let manual_start = TunnelStatus::manual_start(TunnelProvider::CloudflareQuick);

    // When: both statuses are serialized for the status boundary.
    let disabled_json = serde_json::to_value(disabled).unwrap();
    let manual_start_json = serde_json::to_value(manual_start).unwrap();

    // Then: the provider is explicit and the start state is not confused with starting.
    assert_eq!(disabled_json["kind"], "disabled");
    assert_eq!(disabled_json["provider"], "cloudflare_quick");
    assert_eq!(disabled_json["provider_label"], "Cloudflare Quick Tunnel");
    assert_eq!(disabled_json["retryable_error"], false);
    assert_eq!(manual_start_json["kind"], "start");
    assert_eq!(manual_start_json["provider"], "cloudflare_quick");
    assert_eq!(
        manual_start_json["provider_label"],
        "Cloudflare Quick Tunnel"
    );
}

#[test]
fn tunnel_status_serializes_starting_running_and_retryable_error_states() {
    // Given: a single Cloudflare session moving through start, run, and retryable error views.
    let now = fixture_time();
    let session_id = TunnelSessionId::new("session-fixture-1");
    let starting = TunnelStatus::starting(
        TunnelProvider::CloudflareQuick,
        TunnelSession::starting(session_id.clone(), now),
    );
    let running = TunnelStatus::running(
        TunnelProvider::CloudflareQuick,
        TunnelSession::running(session_id.clone(), "https://fixture.trycloudflare.com", now),
    );
    let error = TunnelStatus::retryable_error(
        TunnelProvider::CloudflareQuick,
        "cloudflared exited before URL was reported",
        now,
    );

    // When: the lifecycle statuses are serialized.
    let starting_json = serde_json::to_value(starting).unwrap();
    let running_json = serde_json::to_value(running).unwrap();
    let error_json = serde_json::to_value(error).unwrap();

    // Then: session identity, public URL, checked timestamp, and retryability are distinct fields.
    assert_eq!(starting_json["kind"], "starting");
    assert_eq!(starting_json["session_id"], "session-fixture-1");
    assert_eq!(running_json["kind"], "running");
    assert_eq!(
        running_json["public_url"],
        "https://fixture.trycloudflare.com"
    );
    assert_eq!(running_json["checked_at"], "2026-06-28T12:00:00Z");
    assert_eq!(running_json["checked_at_display"], "2026-06-28T12:00:00Z");
    assert_eq!(error_json["kind"], "error");
    assert_eq!(error_json["retryable_error"], true);
    assert_eq!(
        error_json["message"],
        "cloudflared exited before URL was reported"
    );
}

#[test]
fn tunnel_status_serializes_ssh_as_explicit_unsupported_provider() {
    // Given: the future SSH provider shape exists but is inert in this phase.
    let now = fixture_time();
    let status = TunnelStatus::unsupported(TunnelProvider::Ssh, now);

    // When: the unsupported status is serialized.
    let value = serde_json::to_value(status).unwrap();

    // Then: SSH is represented only as unsupported, not executable provider behavior.
    assert_eq!(value["kind"], "unsupported");
    assert_eq!(value["provider"], "ssh");
    assert_eq!(value["provider_label"], "SSH Tunnel");
    assert_eq!(value["retryable_error"], false);
}

#[test]
fn tunnel_status_rejects_unknown_provider_input() {
    // Given: an unrecognized provider token at the serde boundary.
    let provider = json!("localtunnel");

    // When: the provider is parsed.
    let parsed = serde_json::from_value::<TunnelProvider>(provider);

    // Then: only cloudflare_quick and inert ssh are accepted.
    assert!(parsed.is_err());
}

#[test]
fn tunnel_status_snapshot_serializes_tunnel_separately_from_web_url() {
    // Given: local WebUI URL and public tunnel URL are different surfaces.
    let now = fixture_time();
    let snapshot = AppSnapshot::from_state(
        &SessionState::new(),
        &MissionTracker::new(),
        now,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::running("127.0.0.1:8765", "http://127.0.0.1:8765", now),
    )
    .with_tunnel_status(TunnelStatus::running(
        TunnelProvider::CloudflareQuick,
        TunnelSession::running(
            TunnelSessionId::new("session-fixture-2"),
            "https://fixture.trycloudflare.com",
            now,
        ),
    ));

    // When: the snapshot is serialized.
    let value = serde_json::to_value(snapshot).unwrap();

    // Then: tunnel.public_url is separate and web.url remains the local WebUI URL.
    assert_eq!(value["web"]["url"], "http://127.0.0.1:8765");
    assert_eq!(value["web"].get("public_url"), None);
    assert_eq!(value["tunnel"]["kind"], "running");
    assert_eq!(
        value["tunnel"]["public_url"],
        "https://fixture.trycloudflare.com"
    );
}

#[test]
fn tunnel_status_manager_keeps_ssh_provider_inert_and_unsupported() {
    // Given: the future SSH slot is selected through the provider manager contract.
    let now = fixture_time();
    let mut manager = TunnelManager::new(SshTunnelProvider);

    // When: status, start, and stop are requested for SSH.
    let status = manager.status(now);
    let started = manager.start(now);
    let stopped = manager.stop(now);

    // Then: every operation is explicit unsupported data, not execution behavior.
    assert_eq!(manager.provider(), TunnelProvider::Ssh);
    for value in [status, started, stopped] {
        assert_eq!(value.kind, TunnelStatusKind::Unsupported);
        assert_eq!(value.provider, TunnelProvider::Ssh);
        assert_eq!(value.public_url, None);
        assert_eq!(value.session_id, None);
        assert!(!value.retryable_error);
    }
}

#[test]
fn tunnel_status_manager_exposes_cloudflare_lifecycle_without_spawning() {
    // Given: the Cloudflare Quick provider manager starts from disabled status data.
    let now = fixture_time();
    let mut manager = TunnelManager::new(CloudflareQuickTunnelProvider::disabled());

    // When: the future lifecycle surface is exercised without task-5 process handling.
    let initial = manager.status(now);
    let started = manager.start(now);
    let after_start = manager.status(now);
    let stopped = manager.stop(now);

    // Then: the contract exposes start/stop/status while remaining data-only.
    assert_eq!(manager.provider(), TunnelProvider::CloudflareQuick);
    assert_eq!(initial.kind, TunnelStatusKind::Disabled);
    assert_eq!(started.kind, TunnelStatusKind::Start);
    assert_eq!(after_start.kind, TunnelStatusKind::Start);
    assert_eq!(stopped.kind, TunnelStatusKind::Disabled);
    assert_eq!(started.public_url, None);
    assert_eq!(started.session_id, None);
}

#[test]
#[ignore]
fn tunnel_status_manual_qa_serializes_running_cloudflare_fixture() {
    let now = fixture_time();
    let status = TunnelStatus::running(
        TunnelProvider::CloudflareQuick,
        TunnelSession::running(
            TunnelSessionId::new("manual-qa-session"),
            "https://fixture.trycloudflare.com",
            now,
        ),
    );
    let value = serde_json::to_value(status).unwrap();

    assert_eq!(value["kind"], "running");
    assert_eq!(value["public_url"], "https://fixture.trycloudflare.com");
    println!("{}", serde_json::to_string_pretty(&value).unwrap());
}
