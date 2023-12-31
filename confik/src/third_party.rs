//! Implementations of [`Configuration`](crate::Configuration) for frequently used types from other
//! crates.

#[cfg(feature = "camino")]
mod camino {
    impl crate::Configuration for camino::Utf8PathBuf {
        type Builder = Option<Self>;
    }
}

#[cfg(feature = "chrono")]
mod chrono {
    use chrono::{DateTime, TimeZone};
    use serde::de::DeserializeOwned;

    use crate::Configuration;

    impl<T: TimeZone> Configuration for DateTime<T>
    where
        Self: DeserializeOwned,
    {
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

#[cfg(feature = "rust_decimal")]
mod decimal {
    use rust_decimal::Decimal;

    use crate::Configuration;

    impl Configuration for Decimal {
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
