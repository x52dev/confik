//! Implementations of [`Configuration`](crate::Configuration) for frequently used types from other
//! crates.

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

    use crate::{Configuration, ConfigurationBuilder};

    impl<T> Configuration for JsOption<T>
    where
        T: DeserializeOwned + Configuration,
    {
        type Builder = JsOption<<T as Configuration>::Builder>;
    }

    impl<T> ConfigurationBuilder for JsOption<T>
    where
        T: DeserializeOwned + ConfigurationBuilder,
    {
        type Target = JsOption<<T as ConfigurationBuilder>::Target>;

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
    use confik::std_impls::{KeyedContainerBuilder, UnkeyedContainerBuilder};
    use serde::de::DeserializeOwned;

    use crate::{std_impls::KeyedContainer, Configuration};

    impl<T> Configuration for AHashSet<T>
    where
        T: Configuration + Hash + Eq,
        <T as Configuration>::Builder: Hash + Eq + 'static,
    {
        type Builder = UnkeyedContainerBuilder<AHashSet<<T as Configuration>::Builder>, Self>;
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
        <V as Configuration>::Builder: 'static,
    {
        type Builder = KeyedContainerBuilder<AHashMap<K, <V as Configuration>::Builder>, Self>;
    }
}
