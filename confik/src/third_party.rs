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

#[cfg(feature = "hashbrown")]
mod hashbrown {
    use std::{
        fmt::Display,
        hash::{BuildHasher, Hash},
    };

    use hashbrown::{HashMap, HashSet};
    use serde::de::DeserializeOwned;

    use crate::{
        std_impls::{BuilderOf, KeyedContainer, KeyedContainerBuilder, UnkeyedContainerBuilder},
        Configuration,
    };

    impl<K, V, S> KeyedContainer for HashMap<K, V, S>
    where
        K: Hash + Eq,
        S: BuildHasher + Default,
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

    impl<K, V, S> Configuration for HashMap<K, V, S>
    where
        K: Hash + Eq + Display + DeserializeOwned + 'static,
        V: Configuration,
        BuilderOf<V>: 'static,
        S: Default + BuildHasher + 'static,
    {
        type Builder = KeyedContainerBuilder<HashMap<K, BuilderOf<V>, S>, Self>;
    }

    impl<T, S> Configuration for HashSet<T, S>
    where
        T: Configuration + Eq + Hash,
        BuilderOf<T>: Hash + Eq + 'static,
        S: BuildHasher + Default + 'static,
    {
        type Builder = UnkeyedContainerBuilder<HashSet<BuilderOf<T>, S>, Self>;
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
