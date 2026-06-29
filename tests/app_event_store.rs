use chrono::{TimeZone, Utc};
use ed_sentry::app::{
    AppEventStore, AppSnapshot, MatrixStartupStatus, TunnelProvider, TunnelSession,
    TunnelSessionId, TunnelStatus, TunnelStatusKind, WebStartupStatus,
};
use ed_sentry::mission::MissionTracker;
use ed_sentry::state::SessionState;

#[test]
fn event_store_preserves_checked_tunnel_status_when_runtime_snapshot_has_default_tunnel() {
    // Given: a live event store already contains a manually started running tunnel.
    let initial_time = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 0).unwrap();
    let store = AppEventStore::new(snapshot_at(initial_time));
    let running = TunnelStatus::running(
        TunnelProvider::CloudflareQuick,
        TunnelSession::running(
            TunnelSessionId::new("cloudflare-test-1"),
            "https://fixture.trycloudflare.com",
            initial_time,
        ),
    );
    store.publish_snapshot(snapshot_at(initial_time).with_tunnel_status(running));

    // When: the monitor runtime publishes a fresh snapshot with its default tunnel field.
    let runtime_time = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 1).unwrap();
    store.publish_snapshot(snapshot_at(runtime_time));

    // Then: the checked running tunnel remains visible instead of being overwritten.
    let snapshot = store.subscribe().bootstrap.snapshot;
    assert_eq!(snapshot.tunnel.kind, TunnelStatusKind::Running);
    assert_eq!(
        snapshot.tunnel.public_url.as_deref(),
        Some("https://fixture.trycloudflare.com")
    );
}

fn snapshot_at(now: chrono::DateTime<Utc>) -> AppSnapshot {
    AppSnapshot::from_state(
        &SessionState::default(),
        &MissionTracker::new(),
        now,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    )
}
