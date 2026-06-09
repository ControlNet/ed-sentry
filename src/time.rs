use chrono::{DateTime, Duration, FixedOffset, Local, Utc};

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S %:z";

pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

impl<T: Clock + ?Sized> Clock for &T {
    fn now(&self) -> DateTime<Utc> {
        (*self).now()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedClock {
    current: DateTime<Utc>,
}

impl FixedClock {
    pub fn new(current: DateTime<Utc>) -> Self {
        Self { current }
    }

    pub fn set(&mut self, current: DateTime<Utc>) {
        self.current = current;
    }

    pub fn advance(&mut self, duration: Duration) {
        self.current += duration;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.current
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeDisplayZone {
    Utc,
    Local,
    FixedOffset(FixedOffset),
}

pub fn format_timestamp(timestamp: DateTime<Utc>, zone: TimeDisplayZone) -> String {
    match zone {
        TimeDisplayZone::Utc => timestamp.format(TIMESTAMP_FORMAT).to_string(),
        TimeDisplayZone::Local => timestamp
            .with_timezone(&Local)
            .format(TIMESTAMP_FORMAT)
            .to_string(),
        TimeDisplayZone::FixedOffset(offset) => timestamp
            .with_timezone(&offset)
            .format(TIMESTAMP_FORMAT)
            .to_string(),
    }
}

pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.num_seconds().max(0);

    if total_seconds < 60 {
        return format!("{total_seconds}s");
    }

    let total_minutes = total_seconds / 60;
    if total_minutes < 60 {
        return format!("{total_minutes}m");
    }

    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours}h{minutes}m")
}

#[cfg(test)]
mod time_stats {
    use super::*;
    use chrono::TimeZone;

    fn utc_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 9, 12, 34, 56)
            .single()
            .unwrap()
    }

    #[test]
    fn fixed_clock_returns_configured_time_without_wall_clock() {
        let initial = utc_timestamp();
        let mut clock = FixedClock::new(initial);

        assert_eq!(clock.now(), initial);

        clock.advance(Duration::seconds(90));
        assert_eq!(clock.now(), initial + Duration::seconds(90));

        let replacement = initial + Duration::hours(3);
        clock.set(replacement);
        assert_eq!(clock.now(), replacement);
    }

    #[test]
    fn utc_and_configured_local_display_are_deterministic() {
        let timestamp = utc_timestamp();
        let offset = FixedOffset::east_opt(2 * 60 * 60).unwrap();

        assert_eq!(
            format_timestamp(timestamp, TimeDisplayZone::Utc),
            "2026-06-09 12:34:56 +00:00"
        );
        assert_eq!(
            format_timestamp(timestamp, TimeDisplayZone::FixedOffset(offset)),
            "2026-06-09 14:34:56 +02:00"
        );
    }
}

#[cfg(test)]
mod time_stats_duration_format {
    use super::*;

    #[test]
    fn formats_zero_and_seconds() {
        assert_eq!(format_duration(Duration::zero()), "0s");
        assert_eq!(format_duration(Duration::seconds(58)), "58s");
    }

    #[test]
    fn formats_minutes_and_hours() {
        assert_eq!(format_duration(Duration::minutes(12)), "12m");
        assert_eq!(
            format_duration(Duration::minutes(59) + Duration::seconds(59)),
            "59m"
        );
        assert_eq!(format_duration(Duration::hours(1)), "1h0m");
        assert_eq!(
            format_duration(Duration::hours(3) + Duration::minutes(12)),
            "3h12m"
        );
    }

    #[test]
    fn clamps_negative_durations_to_zero() {
        assert_eq!(format_duration(Duration::seconds(-1)), "0s");
    }
}
