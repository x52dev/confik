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
