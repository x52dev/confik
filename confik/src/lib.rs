//! # `confik`
//!
//! `confik` is a library for reading application configuration split across multiple sources.
//!
//! ## Example
//!
//! Assume that `config.toml` contains:
//!
//! ```toml
//! host=google.com
//! username=root
//! ```
//!
//! and the environment contains:
//!
//! ```bash
//! PASSWORD=hunter2
//! ```
//!
//! then:
//!
//! ```no_run
//! # #[cfg(all(feature = "toml", feature = "env"))]
//! # {
//! use confik::{Configuration, EnvSource, FileSource, TomlSource};
//!
//! #[derive(Debug, PartialEq, Configuration)]
//! struct Config {
//!     host: String,
//!     username: String,
//!
//!     #[confik(secret)]
//!     password: String,
//! }
//!
//! let config = Config::builder()
//!     .override_with(FileSource::new("config.toml"))
//!     .override_with(EnvSource::new().allow_secrets())
//!     .try_build()
//!     .unwrap();
//!
//! assert_eq!(
//!     config,
//!     Config {
//!         host: "google.com".to_string(),
//!         username: "root".to_string(),
//!         password: "hunter2".to_string(),
//!     }
//! );
//! # }
//! ```
//!
//! ## Sources
//!
//! A [`Source`] is any type that can create [`ConfigurationBuilder`]s.
//! This crate implements the following sources:
//!
//! - [`EnvSource`]:
//!   Loads configuration from environment variables using the [`envious`] crate.
//!   Requires the `env` feature. (Enabled by default.)
//! - [`FileSource`]:
//!   Loads configuration from a file, detecting `json` or `toml` files based on the file extension.
//!   Requires the `json` and `toml` feature respectively. (`toml` is enabled by default.)
//! - [`TomlSource`]:
//!   Loads configuration from a TOML string literal.
//!   Requires the `toml` feature. (Enabled by default.)
//! - [`JsonSource`]:
//!   Loads configuration from a JSON string literal.
//!   Requires the `json` feature.
//!
//! ## Secrets
//!
//! Fields annotated with `#[confik(secret)]` will only be read from secure sources.
//! This serves as a runtime check that no secrets have been stored in insecure places such as
//! world-readable files.
//!
//! If a secret is found in an insecure source, an error will be returned.
//! You can opt into loading secrets on a source-by-source basis.
//!
//! ## Foreign Types
//!
//! This crate provides implementations of [`Configuration`] for a number of `std` types and the
//! following third-party crates. Implementations for third-party crates are feature gated.
//!
//! - `with-chrono` - `chrono` 0.4
//! - `with-rust_decimal` - `rust_decimal` 1
//! - `with-url` - `url` 1
//! - `with-uuid` - `uuid` 1
//!
//! ## Macro usage
//!
//! The derive macro is called `Configuration` and is used as normal:
//!
//! ```
//! #[derive(confik::Configuration)]
//! struct Config {
//!     data: usize,
//! }
//! ```
//!
//! ### Forwarding Attributes To `Deserialize`
//!
//! The serde attributes used for customizing a `Deserialize` derive typically are achieved by
//! adding `#[confik(forward_serde(...))` attributes.
//!
//! For example:
//!
//! ```
//! #[derive(confik::Configuration)]
//! struct Config {
//!     #[confik(forward_serde(rename = "other_data"))]
//!     data: usize,
//! }
//! ```
//!
//! ### Defaults
//!
//! Defaults are specified on a per-field basis.
//! * Defaults are used if the data cannot be fully read, even if it is partially read.
//!   E.g., even if `data` in the below example has one value read in, both will be overwritten by
//!   the default.
//!   ```
//!   # #[cfg(feature = "toml")]
//!   # {
//!   use confik::{Configuration, TomlSource};
//!
//!   #[derive(Configuration)]
//!   struct Data {
//!       a: usize,
//!       b: usize,
//!   }
//!
//!   #[derive(Configuration)]
//!   struct Config {
//!       #[confik(default = Data  { a: 1, b: 2 })]
//!       data: Data
//!   }
//!
//!   let toml = r#"
//!       [data]
//!       a = 1234
//!   "#;
//!
//!   let config = Config::builder()
//!       .override_with(TomlSource::new(toml))
//!       .try_build()
//!       .unwrap();
//!   assert_eq!(config.data.a, 1);
//!
//!   let toml = r#"
//!       [data]
//!       a = 1234
//!       b = 4321
//!   "#;
//!
//!   let config = Config::builder()
//!       .override_with(TomlSource::new(toml))
//!       .try_build()
//!       .unwrap();
//!   assert_eq!(config.data.a, 1234);
//!   # }
//!   ```
//! * Defaults can be given by any rust expression, and have [`Into::into`] run over them. E.g.,
//!   ```
//!   const DEFAULT_VALUE: u8 = 4;
//!
//!   #[derive(confik::Configuration)]
//!   struct Config {
//!       #[confik(default = DEFAULT_VALUE)]
//!       a: u32,
//!       #[confik(default = "hello world")]
//!       b: String,
//!       #[confik(default = 5f32)]
//!       c: f32,
//!   }
//!   ```
//! * Alternatively, a default without a given value called [`Default::default`]. E.g.,
//!   ```
//!   use confik::{Configuration};
//!
//!   #[derive(Configuration)]
//!   struct Config {
//!       #[confik(default)]
//!       a: usize
//!   }
//!
//!   let config = Config::builder().try_build().unwrap();
//!   assert_eq!(config.a, 0);
//!   ```
//!
//! ### Handling Foreign Types
//!
//! If there's a foreign type used in your config, then you will not be able to implement
//! [`Configuration`] for it. Instead any type that implements [`Into`] can be used.
//!
//! ```
//! struct ForeignType {
//!     data: usize,
//! }
//!
//! #[derive(confik::Configuration)]
//! struct MyForeignTypeCopy {
//!     data: usize
//! }
//!
//! impl From<MyForeignTypeCopy> for ForeignType {
//!     fn from(copy: MyForeignTypeCopy) -> Self {
//!         Self {
//!             data: copy.data,
//!         }
//!     }
//! }
//!
//! #[derive(confik::Configuration)]
//! struct Config {
//!     #[confik(from = "MyForeignTypeCopy")]
//!     foreign_data: ForeignType
//! }
//! ```
//!
//! ## Macro Limitations
//!
//! ### `Option` Defaulting
//!
//! `Option`s cannot default to anything other than `None`. I.e., the below example ignores the provided default.
//!
//! ```
//! # use confik::Configuration;
//!
//! const DEFAULT_DATA: Option<usize> = Some(5);
//!
//! #[derive(Configuration)]
//! struct Config {
//!     #[confik(default = DEFAULT_DATA)]
//!     data: Option<usize>
//! }
//!
//! let config = Config::builder().try_build().unwrap();
//! assert_eq!(config.data, None);
//! ```
//!
//! This behaviour occurs due to `Option`s needing to have a default value of `None`, as `Option`al
//! configuration shouldn't be required. This defaulting occurs inside the [`ConfigurationBuilder`]
//! implementation of [`Option`] and so happens before the macro can try to default the value.
//!
//! This is in principle fixable by special casing any value with an `Option<...>` type, but this
//! has not been implemented due to the fragility that trying to exact match on the string value of
//! a type in a macro would bring. E.g., a custom type such as `type MyOption = Option<usize>` would
//! then behave differently to using `Option<usize>` directly.
//!
//! ### Custom `Deserialize` Implementations
//!
//! If you're using a custom `Deserialize` implementation, then you cannot use the `Configuration`
//! derive macro. Instead, define the necessary config implementation manually like so:
//!
//! ```rust
//! #[derive(Debug, serde_with::DeserializeFromStr)]
//! enum MyEnum {
//!     Foo,
//!     Bar,
//! };
//!
//! impl std::str::FromStr for MyEnum {
//!     // ...
//! # type Err = String;
//! # fn from_str(_: &str) -> Result<Self, Self::Err> { unimplemented!() }
//! }
//!
//! impl confik::Configuration for MyEnum {
//!     type Builder = Option<Self>;
//! }
//! ```
//!
//! Note that the `Option<Self>` builder type only works for simple types. For more info, see the
//! docs on [`Configuration`] and [`ConfigurationBuilder`].

#![deny(rust_2018_idioms, nonstandard_style, future_incompatible)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::{borrow::Cow, ops::Not};

#[doc(hidden)]
pub use confik_macros::*;

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
    secrets::{SecretBuilder, UnexpectedSecret},
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
pub trait ConfigurationBuilder: Default + serde::de::DeserializeOwned {
    /// The target that will be converted into. See [`Configuration`].
    type Target;

    /// Combines two builders recursively, preferring `self`'s data, if present.
    #[must_use]
    fn merge(self, other: Self) -> Self;

    /// This will probably delegate to `TryInto` but allows it to be implemented for types foreign
    /// to the library.
    fn try_build(self) -> Result<Self::Target, MissingValue>;

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
    T: serde::de::DeserializeOwned + Configuration,
{
    type Target = T;

    fn merge(self, other: Self) -> Self {
        self.or(other)
    }

    fn try_build(self) -> Result<Self::Target, MissingValue> {
        self.ok_or_else(|| MissingValue(Path::new()))
    }

    /// Should not have an `Option` wrapping a secret as `<Option<T> as ConfigurationBuilder` is
    /// used for terminal types, therefore the `SecretBuilder` wrapping would be external to it.
    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        Ok(self.is_some())
    }
}
