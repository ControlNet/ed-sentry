use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::text::format_rate_per_hour;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RateView {
    pub value: f64,
    pub display: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ValueDisplay<T> {
    pub value: T,
    pub display: String,
}

impl RateView {
    pub(crate) fn new(value: f64) -> Self {
        Self {
            value,
            display: format_rate_per_hour(value),
        }
    }
}

pub(crate) fn display_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub(crate) fn credits_u64(value: u64) -> ValueDisplay<u64> {
    ValueDisplay {
        value,
        display: format!("{} cr", format_integer(value)),
    }
}

pub(crate) fn credits_i64(value: i64) -> ValueDisplay<i64> {
    let display = if value < 0 {
        format!("-{} cr", format_integer(value.unsigned_abs()))
    } else {
        format!("{} cr", format_integer(value as u64))
    };
    ValueDisplay { value, display }
}

fn format_integer(value: u64) -> String {
    let digits = value.to_string();
    let mut display = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        let remaining = digits.len() - index;
        display.push(digit);
        if remaining > 1 && remaining % 3 == 1 {
            display.push(',');
        }
    }
    display
}
