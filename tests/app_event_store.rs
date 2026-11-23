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

#[test]
fn event_store_does_not_overwrite_newer_running_tunnel_with_older_error() {
    let older_time = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 0).unwrap();
    let newer_time = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 10).unwrap();
    let later_error_time = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 20).unwrap();
    let store = AppEventStore::new(snapshot_at(older_time));

    let running = TunnelStatus::running(
        TunnelProvider::CloudflareQuick,
        TunnelSession::running(
            TunnelSessionId::new("cloudflare-test-2"),
            "https://fresh.trycloudflare.com",
            newer_time,
        ),
    );
    store.publish_snapshot(snapshot_at(newer_time).with_tunnel_status(running));

    let stale_error = TunnelStatus::retryable_error(
        TunnelProvider::CloudflareQuick,
        "stale startup error",
        older_time,
    );
    store.publish_snapshot(snapshot_at(newer_time).with_tunnel_status(stale_error));

    let after_stale = store.subscribe().bootstrap.snapshot;
    assert_eq!(after_stale.tunnel.kind, TunnelStatusKind::Running);
    assert_eq!(
        after_stale.tunnel.public_url.as_deref(),
        Some("https://fresh.trycloudflare.com")
    );

    let fresh_error = TunnelStatus::retryable_error(
        TunnelProvider::CloudflareQuick,
        "cloudflared exited",
        later_error_time,
    );
    store.publish_snapshot(snapshot_at(later_error_time).with_tunnel_status(fresh_error));

    let after_exit = store.subscribe().bootstrap.snapshot;
    assert_eq!(after_exit.tunnel.kind, TunnelStatusKind::Error);
    assert!(after_exit.tunnel.retryable_error);
}

#[test]
fn event_store_snapshot_reads_without_growing_subscribers() {
    let now = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 0).unwrap();
    let store = AppEventStore::new(snapshot_at(now));

    assert_eq!(store.subscriber_count(), 0);
    let read_one = store.snapshot();
    let read_two = store.snapshot();
    assert_eq!(read_one, read_two);
    assert_eq!(store.subscriber_count(), 0);

    let _subscriber = store.subscribe();
    assert_eq!(store.subscriber_count(), 1);
    let _read_three = store.snapshot();
    assert_eq!(store.subscriber_count(), 1);
}

#[test]
fn event_store_update_tunnel_status_preserves_newer_session_and_broadcasts_only_on_change() {
    let initial_time = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 0).unwrap();
    let update_time = Utc.with_ymd_and_hms(2026, 6, 29, 4, 0, 5).unwrap();
    let store = AppEventStore::new(snapshot_at(initial_time));
    let subscriber = store.subscribe();

    // Caller reads an older snapshot before another task updates session telemetry.
    let stale_before_session_update = store.snapshot();
    assert_eq!(stale_before_session_update.session.kills, 0);

    let mut updated_session = SessionState::default();
    updated_session.kills = 12;
    updated_session.bounty_total = 450_000;
    store.publish_snapshot(AppSnapshot::from_state(
        &updated_session,
        &MissionTracker::new(),
        update_time,
        MatrixStartupStatus::disabled(),
        WebStartupStatus::disabled(),
    ));
    let _ = subscriber.live.try_recv().unwrap();

    // Atomic tunnel update replaces only `tunnel` and keeps the newer session data intact.
    let running = TunnelStatus::running(
        TunnelProvider::CloudflareQuick,
        TunnelSession::running(
            TunnelSessionId::new("cloudflare-test-3"),
            "https://atomic.trycloudflare.com",
            update_time,
        ),
    );
    let after_tunnel = store.update_tunnel_status(running.clone());
    assert_eq!(after_tunnel.session.kills, 12);
    assert_eq!(after_tunnel.session.bounty_total.value, 450_000);
    assert_eq!(after_tunnel.tunnel.kind, TunnelStatusKind::Running);
    assert_eq!(
        after_tunnel.tunnel.public_url.as_deref(),
        Some("https://atomic.trycloudflare.com")
    );
    assert!(subscriber.live.try_recv().is_ok());

    // Re-applying identical tunnel status with a newer timestamp does not emit a redundant broadcast.
    let unchanged_later = TunnelStatus::running(
        TunnelProvider::CloudflareQuick,
        TunnelSession::running(
            TunnelSessionId::new("cloudflare-test-3"),
            "https://atomic.trycloudflare.com",
            update_time + chrono::Duration::seconds(5),
        ),
    );
    let after_unchanged = store.update_tunnel_status(unchanged_later);
    assert_eq!(after_unchanged.session.kills, 12);
    assert!(subscriber.live.try_recv().is_err());

    // Older tunnel status does not overwrite the newer one.
    let stale_error =
        TunnelStatus::retryable_error(TunnelProvider::CloudflareQuick, "stale error", initial_time);
    let after_stale = store.update_tunnel_status(stale_error);
    assert_eq!(after_stale.tunnel.kind, TunnelStatusKind::Running);
    assert!(subscriber.live.try_recv().is_err());
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
