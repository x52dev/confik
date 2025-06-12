//! Utilities for manual implementations of [`Configuration`].
//!
//! Where possible, the derive should be prefered, but sometimes a manual implementation is
//! required.

use std::{fmt::Display, marker::PhantomData};

use serde::{de::DeserializeOwned, Deserialize};

use crate::{Configuration, ConfigurationBuilder, Error, MissingValue, UnexpectedSecret};

/// Type alias for easier usage of [`Configuration`] in complex generic statements
pub type BuilderOf<T> = <T as Configuration>::Builder;

/// Type alias for easier usage of [`KeyedContainerBuilder`] and [`UnkeyedContainerBuilder`] in complex generic statements
pub type ItemOf<C> = <C as IntoIterator>::Item;

/// Type alias for easier usage of [`KeyedContainerBuilder`] in complex generic statements
pub type KeyOf<C> = <C as KeyedContainer>::Key;

/// Type alias for easier usage of [`ConfigurationBuilder`] in complex generic statements
pub type TargetOf<B> = <B as ConfigurationBuilder>::Target;

/// Type alias for easier usage of [`KeyedContainerBuilder`] in complex generic statements
pub type ValueOf<C> = <C as KeyedContainer>::Value;

/// Builder type for unkeyed containers such as [`Vec`] (as opposed to keyed containers like
/// [`HashMap`](std::collections::HashMap)).
///
/// This is not required to be used, but is a convient shortcut for unkeyed container types'
/// implementations.
///
/// For keyed containers, see [`KeyedContainerBuilder`].
///
/// Example usage:
/// ```rust
/// use confik::{
///     helpers::{BuilderOf, UnkeyedContainerBuilder},
///     Configuration,
/// };
/// use serde::Deserialize;
///
/// struct MyVec<T> {
///     // ...
/// #   __: Vec<T>,
/// }
///
/// impl<'de, T> Deserialize<'de> for MyVec<T> {
///     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
///     where
///         D: serde::Deserializer<'de>,
///     {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<T> Default for MyVec<T> {
///     fn default() -> Self {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<T> IntoIterator for MyVec<T> {
///     type Item = T;
///
///     type IntoIter = // ...
/// #       <Vec<T> as IntoIterator>::IntoIter;
///
///     fn into_iter(self) -> Self::IntoIter {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<'a, T> IntoIterator for &'a MyVec<T> {
///     type Item = &'a T;
///
///     type IntoIter = // ...
/// #       <&'a [T] as IntoIterator>::IntoIter;
///
///     fn into_iter(self) -> Self::IntoIter {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<T> FromIterator<T> for MyVec<T> {
///     fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<T> Configuration for MyVec<T>
/// where
///     T: Configuration,
///     BuilderOf<T>: 'static,
/// {
///     type Builder = UnkeyedContainerBuilder<MyVec<BuilderOf<T>>, Self>;
/// }
/// ```
#[derive(Debug, Default, Deserialize, Hash, PartialEq, PartialOrd, Eq, Ord)]
#[serde(from = "Container")]
pub enum UnkeyedContainerBuilder<Container, Target> {
    /// No data has been provided yet.
    ///
    /// Default to `None` but allow overwriting by later [`merge`][ConfigurationBuilder::merge]s.
    #[default]
    Unspecified,

    /// Data has been provided.
    ///
    /// Will not be overwritten by later [`merge`][ConfigurationBuilder::merge]s.
    Some(Container),

    /// Never instantiated, used to hold the [`Target`][ConfigurationBuilder::Target] type.
    _PhantomData(PhantomData<fn() -> Target>),
}

impl<Container, Target> From<Container> for UnkeyedContainerBuilder<Container, Target> {
    fn from(value: Container) -> Self {
        Self::Some(value)
    }
}

impl<Container, Target> ConfigurationBuilder for UnkeyedContainerBuilder<Container, Target>
where
    Self: DeserializeOwned,
    Container: IntoIterator + 'static,
    ItemOf<Container>: ConfigurationBuilder,
    Target: Default + FromIterator<TargetOf<ItemOf<Container>>>,
    for<'a> &'a Container: IntoIterator<Item = &'a ItemOf<Container>>,
{
    type Target = Target;

    fn merge(self, other: Self) -> Self {
        if matches!(self, Self::Unspecified) {
            other
        } else {
            self
        }
    }

    fn try_build(self) -> Result<Self::Target, Error> {
        match self {
            Self::Unspecified => Err(Error::MissingValue(MissingValue::default())),
            Self::Some(val) => val
                .into_iter()
                .map(ConfigurationBuilder::try_build)
                .collect(),
            Self::_PhantomData(_) => unreachable!("PhantomData is never instantiated"),
        }
    }

    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        match self {
            Self::Unspecified => Ok(false),

            // An explicit empty container is counted as as data, overriding any default.
            // If this branch is ever reached, then there is some data, even if it is empty.
            // So always return either an error or `true`.
            Self::Some(val) => val
                .into_iter()
                .map(ConfigurationBuilder::contains_non_secret_data)
                .enumerate()
                .find(|(_index, result)| result.is_err())
                .map(|(index, result)| result.map_err(|err| err.prepend(index.to_string())))
                .unwrap_or(Ok(true)),

            Self::_PhantomData(_) => unreachable!("PhantomData is never instantiated"),
        }
    }
}

/// Trait governing access to keyed containers like [`HashMap`](std::collections::HashMap) (as
/// opposed to unkeyed containers like [`Vec`]).
///
/// This trait purely exists to allow for simple usage of [`KeyedContainerBuilder`]. See the docs
/// there for details.
pub trait KeyedContainer {
    type Key;
    type Value;

    fn insert(&mut self, k: Self::Key, v: Self::Value);
    fn remove(&mut self, k: &Self::Key) -> Option<Self::Value>;
}

/// Builder type for keyed containers, such as [`HashMap`](std::collections::HashMap) (as opposed
/// to unkeyed containers like [`Vec`]). This is not required to be used, but is a convient
/// shortcut for map types' implementations.
///
/// Types using this as their builder must implement [`KeyedContainer`].
///
/// For unkeyed containers, see [`UnkeyedContainerBuilder`].
///
/// Example usage:
/// ```rust
/// use std::fmt::Display;
///
/// use confik::{
///     helpers::{BuilderOf, KeyedContainer, KeyedContainerBuilder},
///     Configuration,
/// };
/// use serde::Deserialize;
///
/// struct MyMap<K, V> {
///     // ...
/// #   __: Vec<(K, V)>,
/// }
///
/// impl<'de, K, V> Deserialize<'de> for MyMap<K, V> {
///     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
///     where
///         D: serde::Deserializer<'de>,
///     {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<K, V> Default for MyMap<K, V> {
///     fn default() -> Self {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<K, V> IntoIterator for MyMap<K, V> {
///     type Item = (K, V);
///
///     type IntoIter = // ...
/// #       <Vec<(K, V)> as IntoIterator>::IntoIter;
///
///     fn into_iter(self) -> Self::IntoIter {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<'a, K, V> IntoIterator for &'a MyMap<K, V> {
///     type Item = (&'a K, &'a V);
///
///     type IntoIter = // ...
/// #       <&'a std::collections::HashMap<K, V> as IntoIterator>::IntoIter;
///
///     fn into_iter(self) -> Self::IntoIter {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<K, V> FromIterator<(K, V)> for MyMap<K, V> {
///     fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
///         // ...
/// #        unimplemented!()
///     }
/// }
///
/// impl<K, V> KeyedContainer for MyMap<K, V> {
///     type Key = K;
///
///     type Value = V;
///
///     fn insert(&mut self, k: Self::Key, v: Self::Value) {
///         // ...
/// #        unimplemented!()
///     }
///
///     fn remove(&mut self, k: &Self::Key) -> Option<Self::Value> {
///         // ...
/// #       unimplemented!()
///     }
/// }
///
/// impl<K, V> Configuration for MyMap<K, V>
/// where
///     K: Display + 'static,
///     V: Configuration,
///     BuilderOf<V>: 'static,
/// {
///     type Builder = KeyedContainerBuilder<MyMap<K, BuilderOf<V>>, Self>;
/// }
/// ```
#[derive(Debug, Default, Deserialize, Hash, PartialEq, PartialOrd, Eq, Ord)]
#[serde(from = "Container")]
pub enum KeyedContainerBuilder<Container, Target> {
    /// No data has been provided yet.
    ///
    /// Default to `None` but allow overwriting by later [`merge`][ConfigurationBuilder::merge]s.
    #[default]
    Unspecified,

    /// Data has been provided.
    ///
    /// Will not be overwritten by later [`merge`][ConfigurationBuilder::merge]s.
    Some(Container),

    /// Never instantiated, used to hold the [`Target`][ConfigurationBuilder::Target] type.
    _PhantomData(PhantomData<fn() -> Target>),
}

impl<Container, Target> From<Container> for KeyedContainerBuilder<Container, Target> {
    fn from(value: Container) -> Self {
        Self::Some(value)
    }
}

impl<Container, Target> ConfigurationBuilder for KeyedContainerBuilder<Container, Target>
where
    Self: DeserializeOwned,
    Container:
        KeyedContainer + IntoIterator<Item = (KeyOf<Container>, ValueOf<Container>)> + 'static,
    KeyOf<Container>: Display,
    ValueOf<Container>: ConfigurationBuilder + 'static,
    Target: Default + FromIterator<(KeyOf<Container>, TargetOf<ValueOf<Container>>)>,
    for<'a> &'a Container: IntoIterator<Item = (&'a KeyOf<Container>, &'a ValueOf<Container>)>,
{
    type Target = Target;

    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::_PhantomData(_), _) | (_, Self::_PhantomData(_)) => {
                unreachable!("PhantomData is never instantiated")
            }
            (Self::Unspecified, other) => other,
            (us, Self::Unspecified) => us,
            (Self::Some(mut us), Self::Some(other)) => {
                for (key, their_val) in other {
                    let val = if let Some(our_val) = us.remove(&key) {
                        our_val.merge(their_val)
                    } else {
                        their_val
                    };

                    us.insert(key, val);
                }

                Self::Some(us)
            }
        }
    }

    fn try_build(self) -> Result<Self::Target, Error> {
        match self {
            Self::Unspecified => Err(Error::MissingValue(MissingValue::default())),
            Self::Some(val) => val
                .into_iter()
                .map(|(key, value)| Ok((key, value.try_build()?)))
                .collect(),
            Self::_PhantomData(_) => unreachable!("PhantomData is never instantiated"),
        }
    }

    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        match self {
            Self::Unspecified => Ok(false),

            // An explicit empty container is counted as as data, overriding any default.
            // If this branch is ever reached, then there is some data, even if it is empty.
            // So always return either an error or `true`.
            Self::Some(val) => val
                .into_iter()
                .map(|(key, value)| (key, value.contains_non_secret_data()))
                .find(|(_key, result)| result.is_err())
                .map(|(key, result)| result.map_err(|err| err.prepend(key.to_string())))
                .unwrap_or(Ok(true)),

            Self::_PhantomData(_) => unreachable!("PhantomData is never instantiated"),
        }
    }
}
