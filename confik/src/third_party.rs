//! Implementations of [`Configuration`](crate::Configuration) for frequently used types from other
//! crates.

#[cfg(feature = "humantime")]
pub mod humantime {
    use std::time::Duration;

    use serde::{Deserializer, Serializer};

    use crate::std_impls::OptionBuilder;

    /// Serde helpers for `Option<Duration>` fields in confik structs.
    ///
    /// Use with `#[confik(forward(serde(with = "confik::humantime::option")))]`.
    pub mod option {
        use super::*;

        pub fn serialize<S>(
            value: &OptionBuilder<Option<Duration>>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let opt = match value {
                OptionBuilder::Unspecified | OptionBuilder::None | OptionBuilder::Some(None) => {
                    None
                }
                OptionBuilder::Some(Some(dur)) => Some(*dur),
            };
            humantime_serde::option::serialize(&opt, serializer)
        }

        pub fn deserialize<'de, D>(d: D) -> Result<OptionBuilder<Option<Duration>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let opt: Option<Duration> = humantime_serde::deserialize(d)?;
            Ok(opt.map(Some).into())
        }
    }
}

#[cfg(feature = "bytesize")]
mod bytesize {
    impl crate::Configuration for bytesize::ByteSize {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "camino")]
mod camino {
    impl crate::Configuration for camino::Utf8PathBuf {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "chrono")]
mod chrono {
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
    use serde::de::DeserializeOwned;

    use crate::Configuration;

    impl<T: TimeZone> Configuration for DateTime<T>
    where
        Self: DeserializeOwned,
    {
        type Builder = Option<Self>;
    }

    impl Configuration for NaiveTime {
        type Builder = Option<Self>;
    }

    impl Configuration for NaiveDate {
        type Builder = Option<Self>;
    }

    impl Configuration for NaiveDateTime {
        type Builder = Option<Self>;
    }

    #[cfg(test)]
    #[cfg(feature = "toml")]
    mod tests {
        use crate::TomlSource;

        #[test]
        fn naive_time_format() {
            use chrono::NaiveTime;

            use crate::Configuration;

            #[derive(Configuration)]
            struct Config {
                time: NaiveTime,
            }

            let toml = r#"
                time = "10:00"
            "#;

            assert_eq!(
                Config::builder()
                    .override_with(TomlSource::new(toml))
                    .try_build()
                    .unwrap()
                    .time,
                NaiveTime::from_hms_opt(10, 0, 0).unwrap()
            );
        }

        #[test]
        fn naive_date_format() {
            use chrono::NaiveDate;

            use crate::Configuration;

            #[derive(Configuration)]
            struct Config {
                date: NaiveDate,
            }

            let toml = r#"
                date = "2013-08-09"
            "#;

            assert_eq!(
                Config::builder()
                    .override_with(TomlSource::new(toml))
                    .try_build()
                    .unwrap()
                    .date,
                NaiveDate::from_ymd_opt(2013, 8, 9).unwrap()
            );
        }
    }
}

#[cfg(feature = "rust_decimal")]
mod decimal {
    use rust_decimal::Decimal;

    use crate::Configuration;

    impl Configuration for Decimal {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "ipnetwork")]
mod ipnetwork {
    use ipnetwork::IpNetwork;

    use crate::Configuration;

    impl Configuration for IpNetwork {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "js_option")]
mod js_option {
    use js_option::JsOption;
    use serde::de::DeserializeOwned;

    use crate::{
        helpers::{BuilderOf, TargetOf},
        Configuration, ConfigurationBuilder,
    };

    impl<T> Configuration for JsOption<T>
    where
        T: DeserializeOwned + Configuration,
    {
        type Builder = JsOption<BuilderOf<T>>;
    }

    impl<T> ConfigurationBuilder for JsOption<T>
    where
        T: DeserializeOwned + ConfigurationBuilder,
    {
        type Target = JsOption<TargetOf<T>>;

        fn merge(self, other: Self) -> Self {
            match (self, other) {
                // If both `Some` then merge the contained builders
                (Self::Some(us), Self::Some(other)) => Self::Some(us.merge(other)),
                // If we don't have a value then always take the other
                (Self::Undefined, other) => other,
                // Either:
                // - We're explicitly `Null`
                // - We're explicitly `Some` and the other is `Undefined` or `Null`
                //
                // In either case, just take our value, which should be preferred to other.
                (us, _) => us,
            }
        }

        fn try_build(self) -> Result<Self::Target, crate::Error> {
            match self {
                Self::Undefined => Ok(Self::Target::Undefined),
                Self::Null => Ok(Self::Target::Null),
                Self::Some(val) => Ok(Self::Target::Some(val.try_build()?)),
            }
        }

        fn contains_non_secret_data(&self) -> Result<bool, crate::UnexpectedSecret> {
            match self {
                Self::Some(data) => data.contains_non_secret_data(),

                // An explicit `Null` is counted as data, overriding any default.
                Self::Null => Ok(true),

                Self::Undefined => Ok(false),
            }
        }
    }
}

#[cfg(feature = "secrecy")]
mod secrecy {
    use secrecy::SecretString;

    use crate::{Configuration, SecretOption};

    impl Configuration for SecretString {
        type Builder = SecretOption<Self>;
    }
}

#[cfg(feature = "url")]
mod url {
    use url::Url;

    use crate::Configuration;

    impl Configuration for Url {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "uuid")]
mod uuid {
    use uuid::Uuid;

    use crate::Configuration;

    impl Configuration for Uuid {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "jiff-0_2")]
pub mod jiff {
    use jiff_0_2::{
        civil::{Date, DateTime, Time},
        SignedDuration, Span, Timestamp, Zoned,
    };

    use crate::Configuration;

    impl Configuration for Timestamp {
        type Builder = Option<Self>;
    }

    impl Configuration for Zoned {
        type Builder = Option<Self>;
    }

    impl Configuration for Date {
        type Builder = Option<Self>;
    }

    impl Configuration for Time {
        type Builder = Option<Self>;
    }

    impl Configuration for DateTime {
        type Builder = Option<Self>;
    }

    impl Configuration for Span {
        type Builder = Option<Self>;
    }

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
                        OptionBuilder::Some(Some(val)) => Some(val.clone()),
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

    /// Generates `pub mod optional { serialize, deserialize }` that adapts
    /// `OptionBuilder<Option<T>>` to/from a jiff serde helper.
    ///
    /// Two arms:
    /// - `fn path, T` — jiff exposes `optional` as a free function (e.g.
    ///   `duration::friendly::compact`); deserialization falls back to standard
    ///   serde (jiff types accept both ISO 8601 and friendly on input).
    /// - `path, T` — jiff exposes `optional` as a module with `serialize` /
    ///   `deserialize` functions (e.g. `timestamp::second`,
    ///   `unsigned_duration`).
    macro_rules! forward_option_builder {
        (fn $($seg:ident)::+, $T:ty) => {
            optional_module!(
                $T,
                jiff_0_2::fmt::serde::$($seg)::+::optional,
                serde::Deserialize::deserialize
            );
        };
        ($($seg:ident)::+, $T:ty) => {
            optional_module!(
                $T,
                jiff_0_2::fmt::serde::$($seg)::+::optional::serialize,
                jiff_0_2::fmt::serde::$($seg)::+::optional::deserialize
            );
        };
    }

    /// Serde helpers for [`Option<jiff_0_2::SignedDuration>`] fields using
    /// jiff's [friendly format](https://docs.rs/jiff/latest/jiff/fmt/friendly/index.html).
    ///
    /// Use with `#[confik(forward(serde(with = "confik::jiff::duration::friendly::compact::optional")))]`.
    pub mod duration {
        pub mod friendly {
            pub mod compact {
                forward_option_builder!(fn duration::friendly::compact, jiff_0_2::SignedDuration);
            }
        }
    }

    /// Serde helpers for [`Option<jiff_0_2::Span>`] fields using jiff's
    /// [friendly format](https://docs.rs/jiff/latest/jiff/fmt/friendly/index.html).
    ///
    /// Use with `#[confik(forward(serde(with = "confik::jiff::span::friendly::compact::optional")))]`.
    pub mod span {
        pub mod friendly {
            pub mod compact {
                forward_option_builder!(fn span::friendly::compact, jiff_0_2::Span);
            }
        }
    }

    /// Serde helpers for [`Option<jiff_0_2::Timestamp>`] fields stored as integers.
    ///
    /// Use these when a timestamp appears as an integer number of seconds,
    /// milliseconds, microseconds, or nanoseconds since the Unix epoch in
    /// the configuration source, rather than as a string.
    ///
    /// Use with `#[confik(forward(serde(with = "confik::jiff::timestamp::<PRECISION>::optional")))]`
    /// where `PRECISION` is `second`, `millisecond`, `microsecond`, or `nanosecond`.
    pub mod timestamp {
        pub mod second {
            forward_option_builder!(timestamp::second, jiff_0_2::Timestamp);
        }
        pub mod millisecond {
            forward_option_builder!(timestamp::millisecond, jiff_0_2::Timestamp);
        }
        pub mod microsecond {
            forward_option_builder!(timestamp::microsecond, jiff_0_2::Timestamp);
        }
        pub mod nanosecond {
            forward_option_builder!(timestamp::nanosecond, jiff_0_2::Timestamp);
        }
    }

    /// Serde helpers for [`Option<std::time::Duration>`] fields using jiff's
    /// duration serialization formats.
    pub mod unsigned_duration {
        forward_option_builder!(unsigned_duration, std::time::Duration);

        /// Friendly compact format.
        ///
        /// Use with `#[confik(forward(serde(with = "confik::jiff::unsigned_duration::friendly::compact::optional")))]`.
        pub mod friendly {
            pub mod compact {
                forward_option_builder!(unsigned_duration::friendly::compact, std::time::Duration);
            }
        }
    }
}

#[cfg(feature = "bigdecimal")]
mod bigdecimal {
    use bigdecimal::BigDecimal;

    use crate::Configuration;

    impl Configuration for BigDecimal {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "ahash")]
mod ahash {
    use std::{fmt::Display, hash::Hash};

    use ahash::{AHashMap, AHashSet};
    use serde::de::DeserializeOwned;

    use crate::{
        helpers::{BuilderOf, KeyedContainer, KeyedContainerBuilder, UnkeyedContainerBuilder},
        Configuration,
    };

    impl<T> Configuration for AHashSet<T>
    where
        T: Configuration + Hash + Eq,
        BuilderOf<T>: Hash + Eq + 'static,
    {
        type Builder = UnkeyedContainerBuilder<AHashSet<BuilderOf<T>>, Self>;
    }

    impl<K, V> KeyedContainer for AHashMap<K, V>
    where
        K: Hash + Eq,
    {
        type Key = K;
        type Value = V;

        fn insert(&mut self, k: Self::Key, v: Self::Value) {
            self.insert(k, v);
        }

        fn remove(&mut self, k: &Self::Key) -> Option<Self::Value> {
            self.remove(k)
        }
    }

    impl<K, V> Configuration for AHashMap<K, V>
    where
        K: Hash + Eq + Display + DeserializeOwned + 'static,
        V: Configuration,
        BuilderOf<V>: 'static,
    {
        type Builder = KeyedContainerBuilder<AHashMap<K, BuilderOf<V>>, Self>;
    }
}

#[cfg(feature = "serde_json")]
mod serde_json {
    use serde_json::Value;

    use crate::{
        helpers::{MergingUnsetBuilder, MergingWithUnset},
        Configuration,
    };

    impl Configuration for Value {
        type Builder = MergingUnsetBuilder<Self>;
    }

    impl MergingWithUnset for Value {
        type Target = Self;

        fn merge(self, other: Self) -> Self {
            match (self, other) {
                (
                    primitive @ (Self::Null | Self::Bool(_) | Self::Number(_) | Self::String(_)),
                    _,
                ) => primitive,
                (Self::Array(mut me), Self::Array(other)) => {
                    me.extend(other);
                    Self::Array(me)
                }
                (arr @ Self::Array(_), _) => arr,
                (Self::Object(mut me), Self::Object(other)) => {
                    me.extend(other);
                    Self::Object(me)
                }
                (obj @ Self::Object(_), _) => obj,
            }
        }

        fn try_build(self) -> Result<Self::Target, crate::Error> {
            Ok(self)
        }

        fn contains_non_secret_data(&self) -> Result<bool, crate::UnexpectedSecret> {
            Ok(match self {
                Self::Null => false,
                Self::Array(arr) => !arr.is_empty(),
                Self::Object(map) => !map.is_empty(),
                Self::String(s) => !s.is_empty(),
                Self::Bool(_) | Self::Number(_) => true,
            })
        }
    }
}
