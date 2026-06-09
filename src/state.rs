use chrono::{DateTime, Duration, Utc};

pub const RECENT_RATE_WINDOW: Duration = Duration::minutes(10);

const MINIMUM_TOTAL_RATE_DURATION: Duration = Duration::seconds(1);

pub fn total_kill_rate_per_hour(
    kills: u64,
    session_started_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> f64 {
    total_event_rate_per_hour(kills, session_started_at, now)
}

pub fn total_scan_rate_per_hour(
    scans: u64,
    session_started_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> f64 {
    total_event_rate_per_hour(scans, session_started_at, now)
}

pub fn total_event_rate_per_hour(
    events: u64,
    session_started_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> f64 {
    let Some(started_at) = session_started_at else {
        return 0.0;
    };

    let elapsed = now.signed_duration_since(started_at);
    rate_per_hour(events, elapsed.max(MINIMUM_TOTAL_RATE_DURATION))
}

pub fn recent_kill_rate_per_hour(kill_timestamps: &[DateTime<Utc>], now: DateTime<Utc>) -> f64 {
    recent_event_rate_per_hour(kill_timestamps, now)
}

pub fn recent_scan_rate_per_hour(scan_timestamps: &[DateTime<Utc>], now: DateTime<Utc>) -> f64 {
    recent_event_rate_per_hour(scan_timestamps, now)
}

pub fn recent_event_rate_per_hour(event_timestamps: &[DateTime<Utc>], now: DateTime<Utc>) -> f64 {
    let window_start = now - RECENT_RATE_WINDOW;
    let events_in_window = event_timestamps
        .iter()
        .filter(|timestamp| **timestamp >= window_start && **timestamp <= now)
        .count() as u64;

    rate_per_hour(events_in_window, RECENT_RATE_WINDOW)
}

fn rate_per_hour(events: u64, duration: Duration) -> f64 {
    if events == 0 {
        return 0.0;
    }

    let duration_hours = duration.num_milliseconds() as f64 / 3_600_000.0;
    events as f64 / duration_hours
}

#[cfg(test)]
mod time_stats_rates {
    use super::*;
    use chrono::TimeZone;

    fn utc_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 9, 12, 0, 0).single().unwrap()
    }

    #[test]
    fn inactive_session_total_rates_are_zero() {
        let now = utc_timestamp();

        assert_eq!(total_kill_rate_per_hour(10, None, now), 0.0);
        assert_eq!(total_scan_rate_per_hour(20, None, now), 0.0);
    }

    #[test]
    fn zero_duration_total_rate_is_finite() {
        let now = utc_timestamp();

        let rate = total_kill_rate_per_hour(2, Some(now), now);

        assert!(rate.is_finite());
        assert_eq!(rate, 7200.0);
    }

    #[test]
    fn one_hour_total_rates_match_counts() {
        let now = utc_timestamp();
        let started_at = now - Duration::hours(1);

        assert_eq!(total_kill_rate_per_hour(42, Some(started_at), now), 42.0);
        assert_eq!(total_scan_rate_per_hour(120, Some(started_at), now), 120.0);
    }

    #[test]
    fn ten_minute_recent_rate_uses_rolling_window() {
        let now = utc_timestamp();
        let timestamps = vec![
            now - Duration::minutes(11),
            now - Duration::minutes(10),
            now - Duration::minutes(9),
            now,
            now + Duration::seconds(1),
        ];

        assert_eq!(recent_kill_rate_per_hour(&timestamps, now), 18.0);
        assert_eq!(recent_scan_rate_per_hour(&timestamps, now), 18.0);
    }
}
