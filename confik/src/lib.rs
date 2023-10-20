#![doc = include_str!("./lib.md")]
#![deny(rust_2018_idioms, nonstandard_style, future_incompatible)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::{borrow::Cow, error::Error as StdError, ops::Not};

#[doc(hidden)]
pub use confik_macros::*;
use serde::de::DeserializeOwned;

use crate::{path::Path, sources::DynSource};

#[doc(hidden)]
pub mod __exports {
    /// Re-export [`Deserialize`] for use in case `serde` is not otherwise used
    /// whilst we are.
    ///
    /// As serde then calls into other serde functions, we need to re-export the whole of serde,
    /// instead of just [`Deserialize`].
    ///
    /// [`Deserialize`]: serde::Deserialize
    pub use serde as __serde;
}

// Enable use of macros inside the crate
#[allow(unused_extern_crates)] // false positive
extern crate self as confik;

mod builder;
#[cfg(feature = "common")]
pub mod common;
mod errors;
mod path;
mod secrets;
mod sources;
mod std_impls;
mod third_party;

#[cfg(feature = "env")]
pub use self::sources::env_source::EnvSource;
#[cfg(feature = "json")]
pub use self::sources::json_source::JsonSource;
#[cfg(feature = "toml")]
pub use self::sources::toml_source::TomlSource;
pub use self::{
    builder::ConfigBuilder,
    errors::Error,
    secrets::{SecretBuilder, SecretOption, UnexpectedSecret},
    sources::{file_source::FileSource, Source},
};

/// Captures the path of a missing value.
#[derive(Debug, Default, thiserror::Error)]
#[error("Missing value for path `{0}`")]
pub struct MissingValue(Path);

impl MissingValue {
    /// Prepends a path segment as we return back up the call-stack.
    #[must_use]
    pub fn prepend(mut self, path_segment: impl Into<Cow<'static, str>>) -> Self {
        self.0 .0.push(path_segment.into());
        self
    }
}

/// Captures the path and error of a failed conversion.
#[derive(Debug, thiserror::Error)]
#[error("Failed try_into for path `{0}`: {1}")]
pub struct FailedTryInto(Path, #[source] Box<dyn StdError>);

impl FailedTryInto {
    /// Creates a new [`Self`] with a blank path.
    pub fn new(err: impl StdError + 'static) -> Self {
        Self(Path::new(), Box::new(err))
    }

    /// Prepends a path segment as we return back up the call-stack.
    #[must_use]
    pub fn prepend(mut self, path_segment: impl Into<Cow<'static, str>>) -> Self {
        self.0 .0.push(path_segment.into());
        self
    }
}

/// Converts the sources, in order, into [`Configuration::Builder`] and
/// [`ConfigurationBuilder::merge`]s them, passing any errors back.
fn build_from_sources<'a, Target, Iter>(sources: Iter) -> Result<Target, Error>
where
    Target: Configuration,
    Iter: IntoIterator<Item = Box<dyn DynSource<Target::Builder> + 'a>>,
{
    sources
        .into_iter()
        // Convert each source to a `Target::Builder`
        .map::<Result<Target::Builder, Error>, _>(|s: Box<dyn DynSource<Target::Builder> + 'a>| {
            let debug = || format!("{:?}", s);
            let res = s.provide().map_err(|e| Error::Source(e, debug()))?;
            if s.allows_secrets().not() {
                res.contains_non_secret_data()
                    .map_err(|e| Error::UnexpectedSecret(e, debug()))?;
            }
            Ok(res)
        })
        // Merge the builders
        .reduce(|first, second| Ok(Target::Builder::merge(first?, second?)))
        // If there was no data then we're missing values
        .ok_or_else(|| Error::MissingValue(MissingValue::default()))??
        .try_build()
        .map_err(Into::into)
}

/// The target to be deserialized from multiple sources.
///
/// This will normally be created by the derive macro which also creates a [`ConfigurationBuilder`]
/// implementation.
///
/// For types with no contents, e.g. empty structs, or simple enums, this can be implemented very
/// easily by specifying only the builder type as `Option<Self>`. For anything more complicated,
/// complete target and builder implementations will be needed.
///
/// # Examples
///
/// ```
/// use confik::Configuration;
///
/// #[derive(serde::Deserialize)]
/// enum MyEnum { A, B, C }
///
/// impl Configuration for MyEnum {
///     type Builder = Option<Self>;
/// }
/// ```
pub trait Configuration: Sized {
    /// The builder that accumulates the deserializations.
    type Builder: ConfigurationBuilder<Target = Self>;

    /// Creates an instance of [`ConfigBuilder`] tied to this type.
    #[must_use]
    fn builder<'a>() -> ConfigBuilder<'a, Self> {
        ConfigBuilder::<Self>::default()
    }
}

/// A builder for a multi-source config deserialization.
///
/// This will almost never be implemented manually, instead being derived.
///
/// Builders must implement [`Default`] so that if the structure is nested in another then it being
/// missing is not an error.
/// For trivial cases, this is solved by using an `Option<Configuration>`.
/// See the worked example on [`Configuration`].
pub trait ConfigurationBuilder: Default + DeserializeOwned {
    /// The target that will be converted into. See [`Configuration`].
    type Target;

    /// Combines two builders recursively, preferring `self`'s data, if present.
    #[must_use]
    fn merge(self, other: Self) -> Self;

    /// This will probably delegate to `TryInto` but allows it to be implemented for types foreign
    /// to the library.
    fn try_build(self) -> Result<Self::Target, Error>;

    /// Called recursively on each field, aiming to hit all [`SecretBuilder`]s. This is only called
    /// when [`Source::allows_secrets`] is `false`.
    ///
    /// If any data is present then `Ok(true)` is returned, unless the data is wrapped in a
    /// [`SecretBuilder`] in which case [`UnexpectedSecret`] is passed, which will then be built
    /// into the path to the secret data.
    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret>;
}

/// Implementations for trivial types via `Option`.
///
/// This can also be used for user types, such as an `enum` with no variants containing fields. See
/// the worked example on [`Configuration`].
impl<T> ConfigurationBuilder for Option<T>
where
    T: DeserializeOwned + Configuration,
{
    type Target = T;

    fn merge(self, other: Self) -> Self {
        self.or(other)
    }

    fn try_build(self) -> Result<Self::Target, Error> {
        self.ok_or_else(|| Error::MissingValue(MissingValue::default()))
    }

    /// Should not have an `Option` wrapping a secret as `<Option<T> as ConfigurationBuilder` is
    /// used for terminal types, therefore the `SecretBuilder` wrapping would be external to it.
    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        Ok(self.is_some())
    }
}
