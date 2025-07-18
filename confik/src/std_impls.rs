//! Implementations of [`Configuration`](crate::Configuration) for standard library types.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    ffi::OsString,
    fmt::Display,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::PathBuf,
    sync::atomic::{
        AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicU16, AtomicU32,
        AtomicU64, AtomicU8, AtomicUsize,
    },
    time::{Duration, SystemTime},
};

use serde::{de::DeserializeOwned, Deserialize};

use crate::{
    helpers::{
        BuilderOf, KeyedContainer, KeyedContainerBuilder, TargetOf, UnkeyedContainerBuilder,
    },
    Configuration, ConfigurationBuilder, Error, MissingValue, UnexpectedSecret,
};

/// Convenience macro for the large number of foreign library types to implement the
/// [`Configuration`] using an [`Option`] as their [`ConfigurationBuilder`].
macro_rules! impl_multi_source_via_option {
    ($type:ty) => {
        impl Configuration for $type {
            type Builder = Option<Self>;
        }
    };

    ($($type:ty),* $(,)?) => {
        $(
            impl_multi_source_via_option! { $type }
        )*
    };
}

impl_multi_source_via_option! {
    // Signed integers
    i8, i16, i32, i64, i128, isize,

    // Unsigned integers
    u8, u16, u32, u64, u128, usize,

    // Floats
    f32, f64,

    // Networking types
    SocketAddr, SocketAddrV4, SocketAddrV6, IpAddr, Ipv4Addr, Ipv6Addr,

    // Time
    Duration, SystemTime,

    // Other standard types
    String, OsString, PathBuf, char, bool,

    // Atomic types
    AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize,
    AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize,
    AtomicBool,
}

// Containers
impl<T> Configuration for Vec<T>
where
    T: Configuration,
    BuilderOf<T>: 'static,
{
    type Builder = UnkeyedContainerBuilder<Vec<BuilderOf<T>>, Self>;
}

impl<T> Configuration for BTreeSet<T>
where
    T: Configuration + Ord,
    BuilderOf<T>: Ord + 'static,
{
    type Builder = UnkeyedContainerBuilder<BTreeSet<BuilderOf<T>>, Self>;
}

impl<T, S> Configuration for HashSet<T, S>
where
    T: Configuration + Eq + Hash,
    BuilderOf<T>: Hash + Eq + 'static,
    S: BuildHasher + Default + 'static,
{
    type Builder = UnkeyedContainerBuilder<HashSet<BuilderOf<T>, S>, Self>;
}

impl<K, V> KeyedContainer for BTreeMap<K, V>
where
    K: Ord,
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

impl<K, V> Configuration for BTreeMap<K, V>
where
    K: Ord + Display + DeserializeOwned + 'static,
    V: Configuration,
    BuilderOf<V>: 'static,
{
    type Builder = KeyedContainerBuilder<BTreeMap<K, BuilderOf<V>>, Self>;
}

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

impl<T, const N: usize> Configuration for [T; N]
where
    [BuilderOf<T>; N]: DeserializeOwned + Default,
    T: Configuration,
{
    type Builder = [BuilderOf<T>; N];
}

impl<T, const N: usize> ConfigurationBuilder for [T; N]
where
    Self: DeserializeOwned + Default,
    T: ConfigurationBuilder,
{
    type Target = [TargetOf<T>; N];

    fn merge(self, other: Self) -> Self {
        let mut iter = other.into_iter();
        self.map(|us| us.merge(iter.next().unwrap()))
    }

    fn try_build(self) -> Result<Self::Target, Error> {
        self.into_iter()
            .enumerate()
            .map(|(index, val)| {
                val.try_build().map_err(|err| match err {
                    Error::MissingValue(err) => Error::MissingValue(err.prepend(index.to_string())),
                    err => err,
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|vec: Vec<_>| {
                Error::MissingValue(MissingValue::default().prepend(vec.len().to_string()))
            })
    }

    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        self.iter()
            .map(ConfigurationBuilder::contains_non_secret_data)
            .enumerate()
            .try_fold(false, |has_secret, (index, val)| {
                Ok(val.map_err(|err| err.prepend(index.to_string()))? || has_secret)
            })
    }
}

/// `PhantomData` does not need a builder, however we cannot use `()` as that would make `T`
/// unconstrained. Instead just making it use itself as a builder and rely on serde handling it
/// alright.
impl<T> Configuration for PhantomData<T> {
    type Builder = Self;
}

/// `PhantomData` does not need a builder, however we cannot use `()` as that would make `T`
/// unconstrained. Instead just making it use itself as a builder and rely on serde handling it
/// alright.
impl<T> ConfigurationBuilder for PhantomData<T> {
    type Target = Self;

    fn merge(self, _other: Self) -> Self {
        self
    }

    fn try_build(self) -> Result<Self::Target, Error> {
        Ok(self)
    }

    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        Ok(false)
    }
}

/// Build an `Option<T>` with a custom structure as we want `None` to be an explicit value that will
/// not be overwritten.
impl<T: Configuration> Configuration for Option<T>
where
    OptionBuilder<BuilderOf<T>>: DeserializeOwned,
{
    type Builder = OptionBuilder<BuilderOf<T>>;
}

/// Build an `Option<T>` with a custom structure as we want `None` to be an explicit value that will
/// not be overwritten.
#[derive(Debug, Default, Deserialize, Hash, PartialEq, PartialOrd, Eq, Ord)]
#[serde(from = "Option<T>")]
pub enum OptionBuilder<T> {
    /// No item has been provided yet.
    ///
    /// Default to `None` but allow overwriting by later [`merge`][ConfigurationBuilder::merge]s.
    #[default]
    Unspecified,

    /// Explicit `None`.
    ///
    /// Will not be overwritten by later [`merge`][ConfigurationBuilder::merge]s.
    None,

    /// Explicit `Some`.
    ///
    /// Will not be overwritten by later [`merge`][ConfigurationBuilder::merge]s.
    Some(T),
}

impl<T> From<Option<T>> for OptionBuilder<T> {
    fn from(opt: Option<T>) -> Self {
        opt.map_or(Self::None, |val| Self::Some(val))
    }
}

impl<T: ConfigurationBuilder> ConfigurationBuilder for OptionBuilder<T>
where
    Self: DeserializeOwned,
{
    type Target = Option<TargetOf<T>>;

    fn merge(self, other: Self) -> Self {
        match (self, other) {
            // If both `Some` then merge the contained builders
            (Self::Some(us), Self::Some(other)) => Self::Some(us.merge(other)),
            // If we don't have a value then always take the other
            (Self::Unspecified, other) => other,
            // Either:
            // - We're explicitly `None`
            // - We're explicitly `Some` and the other is `Unspecified` or `None`
            //
            // In either case, just take our value, which should be preferred to other.
            (us, _) => us,
        }
    }

    fn try_build(self) -> Result<Self::Target, Error> {
        match self {
            Self::Unspecified | Self::None => Ok(None),
            Self::Some(val) => Ok(Some(val.try_build()?)),
        }
    }

    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        match self {
            Self::Some(data) => data.contains_non_secret_data(),

            // An explicit `None` is counted as data, overriding any default.
            Self::None => Ok(true),

            Self::Unspecified => Ok(false),
        }
    }
}
