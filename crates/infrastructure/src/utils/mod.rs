pub mod testing;

use chrono::{DateTime, Utc};
use sqlx::types::{time::OffsetDateTime, BigDecimal};
use std::str::FromStr;

/// Convert chrono::DateTime<Utc> to sqlx::types::time::OffsetDateTime
pub fn to_offset_datetime(dt: DateTime<Utc>) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(dt.timestamp()).unwrap()
}

/// Convert Option<chrono::DateTime<Utc>> to Option<sqlx::types::time::OffsetDateTime>
pub fn to_option_offset_datetime(dt: Option<DateTime<Utc>>) -> Option<OffsetDateTime> {
    dt.map(to_offset_datetime)
}

/// Convert sqlx::types::time::OffsetDateTime to chrono::DateTime<Utc>
pub fn from_offset_datetime(dt: Option<OffsetDateTime>) -> DateTime<Utc> {
    dt.map_or_else(Utc::now, |dt| {
        DateTime::<Utc>::from_timestamp(dt.unix_timestamp(), 0).unwrap()
    })
}

/// Convert Option<sqlx::types::time::OffsetDateTime> to Option<chrono::DateTime<Utc>>
pub fn from_option_offset_datetime(dt: Option<OffsetDateTime>) -> Option<DateTime<Utc>> {
    dt.map(|odt| DateTime::<Utc>::from_timestamp(odt.unix_timestamp(), 0).unwrap())
}

/// Convert Option<f64> to Option<BigDecimal>
pub fn to_option_bigdecimal(value: Option<f64>) -> Option<BigDecimal> {
    value.map(|v| BigDecimal::from_str(&v.to_string()).unwrap())
}

/// Convert Option<BigDecimal> to Option<f64>
pub fn from_option_bigdecimal(value: Option<BigDecimal>) -> Option<f64> {
    value.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
}
