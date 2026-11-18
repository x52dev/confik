//! Serde helpers for optional Jiff 1 configuration fields.

use ::jiff_1::{
    civil::{Date, DateTime, Time},
    SignedDuration, Span, Timestamp, ZonedDateTime,
};

use crate::Configuration;

/// Accepts RFC 3339 date-time strings with `Z` or a numeric UTC offset, such as
/// `"2024-01-01T00:00:00Z"` or `"2024-01-01T01:00:00+01:00"`.
impl Configuration for Timestamp {
    type Builder = Option<Self>;
}

/// Accepts RFC 9557 date-time strings with a bracketed time zone, such as
/// `"2024-01-01T00:00:00+00:00[UTC]"`. The numeric UTC offset is optional.
impl Configuration for ZonedDateTime {
    type Builder = Option<Self>;
}

/// Accepts ISO 8601 calendar date strings such as `"2024-01-01"`.
impl Configuration for Date {
    type Builder = Option<Self>;
}

/// Accepts ISO 8601 time strings such as `"10:30:00"` or `"10:30:00.123"`.
impl Configuration for Time {
    type Builder = Option<Self>;
}

/// Accepts ISO 8601 local date-time strings such as `"2024-01-01T10:30:00"`.
impl Configuration for DateTime {
    type Builder = Option<Self>;
}

/// Accepts calendar and clock units in ISO 8601 duration strings such as
/// `"P2D"` and Jiff's friendly duration strings such as `"1d"` or `"2 days"`.
impl Configuration for Span {
    type Builder = Option<Self>;
}

/// Accepts elapsed durations in ISO 8601 strings such as `"PT24H"` and Jiff's
/// friendly strings such as `"24h"` or `"-2h 30m"`. The largest unit is hours.
impl Configuration for SignedDuration {
    type Builder = Option<Self>;
}

/// Generates `pub mod optional { serialize, deserialize }` given explicit
/// serialize and deserialize paths.
macro_rules! optional_module {
    ($T:ty, $serialize_fn:path, $deserialize_fn:path) => {
        pub mod optional {
            use serde::{Deserializer, Serializer};

            use crate::std_impls::OptionBuilder;

            pub fn serialize<S>(
                value: &OptionBuilder<Option<$T>>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let opt = match value {
                    OptionBuilder::Unspecified
                    | OptionBuilder::None
                    | OptionBuilder::Some(None) => None,
                    OptionBuilder::Some(Some(val)) => Some(*val),
                };

                $serialize_fn(&opt, serializer)
            }

            pub fn deserialize<'de, D>(d: D) -> Result<OptionBuilder<Option<$T>>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let opt: Option<$T> = $deserialize_fn(d)?;
                Ok(opt.map(Some).into())
            }
        }
    };
}

/// Adapts `OptionBuilder<Option<T>>` to a Jiff Serde helper.
///
/// Some Jiff helpers only implement serialization. Their types use standard
/// Serde deserialization, which accepts ISO 8601 and friendly formats.
macro_rules! forward_option_builder {
    (fn $($seg:ident)::+, $T:ty) => {
        optional_module!(
            $T,
            ::jiff_1::fmt::serde::$($seg)::+::optional,
            serde::Deserialize::deserialize
        );
    };
    ($($seg:ident)::+, $T:ty) => {
        optional_module!(
            $T,
            ::jiff_1::fmt::serde::$($seg)::+::optional::serialize,
            ::jiff_1::fmt::serde::$($seg)::+::optional::deserialize
        );
    };
}

/// Serde helpers for [`Option<SignedDuration>`](SignedDuration) fields using
/// Jiff's [friendly format](https://docs.rs/jiff/1/jiff/fmt/friendly/index.html).
///
/// Use with `#[confik(forward(serde(with = "confik::jiff_1::duration::friendly::compact::optional")))]`.
pub mod duration {
    pub mod friendly {
        pub mod compact {
            forward_option_builder!(fn duration::friendly::compact, ::jiff_1::SignedDuration);
        }
    }
}

/// Serde helpers for [`Option<Span>`](Span) fields using Jiff's
/// [friendly format](https://docs.rs/jiff/1/jiff/fmt/friendly/index.html).
///
/// Use with `#[confik(forward(serde(with = "confik::jiff_1::span::friendly::compact::optional")))]`.
pub mod span {
    pub mod friendly {
        pub mod compact {
            forward_option_builder!(fn span::friendly::compact, ::jiff_1::Span);
        }
    }
}

/// Serde helpers for [`Option<Timestamp>`](Timestamp) fields stored as integers.
///
/// Use these when a timestamp appears as an integer number of seconds,
/// milliseconds, microseconds, or nanoseconds since the Unix epoch in
/// the configuration source.
///
/// Use with `#[confik(forward(serde(with = "confik::jiff_1::timestamp::<PRECISION>::optional")))]`
/// where `PRECISION` is `second`, `millisecond`, `microsecond`, or `nanosecond`.
pub mod timestamp {
    pub mod second {
        forward_option_builder!(timestamp::second, ::jiff_1::Timestamp);
    }
    pub mod millisecond {
        forward_option_builder!(timestamp::millisecond, ::jiff_1::Timestamp);
    }
    pub mod microsecond {
        forward_option_builder!(timestamp::microsecond, ::jiff_1::Timestamp);
    }
    pub mod nanosecond {
        forward_option_builder!(timestamp::nanosecond, ::jiff_1::Timestamp);
    }
}

/// Serde helpers for [`Option<std::time::Duration>`] fields using Jiff's
/// duration serialization formats.
pub mod unsigned_duration {
    forward_option_builder!(unsigned_duration, std::time::Duration);

    /// Friendly compact format.
    ///
    /// Use with `#[confik(forward(serde(with = "confik::jiff_1::unsigned_duration::friendly::compact::optional")))]`.
    pub mod friendly {
        pub mod compact {
            forward_option_builder!(unsigned_duration::friendly::compact, std::time::Duration);
        }
    }
}
