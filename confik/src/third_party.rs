//! Implementations of [`Configuration`](crate::Configuration) for frequently used types
//! from other crates.

#[cfg(feature = "with-chrono")]
mod chrono {
    use chrono_04::{DateTime, TimeZone};
    use serde::de::DeserializeOwned;

    use crate::Configuration;

    impl<T: TimeZone> Configuration for DateTime<T>
    where
        Self: DeserializeOwned,
    {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "with-rust_decimal-1")]
mod decimal {
    use rust_decimal_1::Decimal;

    use crate::Configuration;

    impl Configuration for Decimal {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "with-url")]
mod url {
    use url_2::Url;

    use crate::Configuration;

    impl Configuration for Url {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "with-uuid")]
mod uuid {
    use uuid_1::Uuid;

    use crate::Configuration;

    impl Configuration for Uuid {
        type Builder = Option<Self>;
    }
}
