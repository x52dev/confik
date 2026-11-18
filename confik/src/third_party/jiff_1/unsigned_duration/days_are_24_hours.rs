//! Serde helpers that treat each day as 24 hours and each week as seven days.
//!
//! Accepts ISO 8601 strings such as `"P2D"` and Jiff's
//! [friendly duration strings](https://docs.rs/jiff/1/jiff/fmt/friendly/index.html)
//! such as `"2 days"` or `"1d 12h"`. Use whole numbers for days and weeks.
//! Months, years, and negative durations return errors. Zero is valid.
//!
//! These helpers preserve subsecond precision. They serialize the resulting
//! elapsed time in ISO 8601 format using hours, minutes, and seconds.
//!
//! # Examples
//!
//! Use [`required`] for a `Duration` field and [`optional`] for an
//! `Option<Duration>` field.
//!
//! ```
//! # #[cfg(feature = "toml")]
//! # {
//! use std::time::Duration;
//!
//! use confik::{Configuration, TomlSource};
//!
//! #[derive(Configuration)]
//! struct Config {
//!     #[confik(forward(serde(
//!         with = "confik::jiff_1::unsigned_duration::days_are_24_hours::required"
//!     )))]
//!     timeout: Duration,
//!     #[confik(forward(serde(
//!         with = "confik::jiff_1::unsigned_duration::days_are_24_hours::optional"
//!     )))]
//!     retry_delay: Option<Duration>,
//! }
//!
//! let config = Config::builder()
//!     .override_with(TomlSource::new(r#"
//!         timeout = "2 days"
//!         retry_delay = "1d 12h"
//!     "#))
//!     .try_build()
//!     .unwrap();
//!
//! assert_eq!(config.timeout, Duration::from_secs(172_800));
//! assert_eq!(config.retry_delay, Some(Duration::from_secs(129_600)));
//! # }
//! ```

use std::time::Duration;

use ::jiff_1::{Span, SpanRelativeTo};
use serde::{de, Deserialize, Deserializer};

fn to_duration(span: Span) -> Result<Duration, ::jiff_1::Error> {
    Duration::try_from(span.to_duration(SpanRelativeTo::days_are_24_hours())?)
}

/// Serde helpers for required [`std::time::Duration`] fields.
pub mod required {
    use std::time::Duration;

    use ::jiff_1::Span;
    use serde::{de, Deserialize, Deserializer, Serializer};

    use super::to_duration;

    /// Serialize a duration as an ISO 8601 string.
    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(duration) => {
                ::jiff_1::fmt::serde::unsigned_duration::required::serialize(duration, serializer)
            }
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize a duration string with 24-hour days.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let span = Span::deserialize(deserializer)?;

        to_duration(span).map(Some).map_err(de::Error::custom)
    }
}

fn deserialize_optional<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Span>::deserialize(deserializer)?
        .map(to_duration)
        .transpose()
        .map_err(de::Error::custom)
}

optional_module!(
    /// Serde helpers for optional [`std::time::Duration`] fields.
    std::time::Duration,
    ::jiff_1::fmt::serde::unsigned_duration::optional::serialize,
    super::deserialize_optional
);
