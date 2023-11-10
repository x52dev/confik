use std::{error::Error, fmt::Debug};

use crate::ConfigurationBuilder;

/// A source of configuration data.
pub trait Source: Debug {
    /// Whether this source is allowed to contain secret data.
    ///
    /// Implementations should be conservative and return `false` by default, allowing users to
    /// opt-in to storing secrets in this source.
    fn allows_secrets(&self) -> bool {
        false
    }

    /// Attempts to provide a partial configuration object from this source.
    fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>>;
}

pub(crate) trait DynSource<T>: Debug {
    fn allows_secrets(&self) -> bool;
    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>>;
}

impl<S, T> DynSource<T> for S
where
    S: Source,
    T: ConfigurationBuilder,
{
    fn allows_secrets(&self) -> bool {
        <S as Source>::allows_secrets(self)
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        <S as Source>::provide(self)
    }
}

/// A source to fallback to the provided `Default::default()` implementation, where one exists.
///
/// This will take priority over any `#[confik(default = ...)]`. The main usecase for this would
/// be where the default values for a set of fields require some processing and so you might end up
/// with a struct like:
/// ```
/// #[derive(Configuration)]
/// struct Config {
///     #[confik(default = "Config::default().field_1")]
///     field_1: usize,
///     #[confik(default = "Config::default().field_2")]
///     field_2: usize,
///     #[confik(default = "Config::default().field_3")]
///     field_3: usize,
/// }
/// ```
///
/// Otherwise it is recommended to use `#[confik(default = ...)]` for its finer grained control.
#[derive(Debug)]
pub struct DefaultSource;

impl<T> DynSource<T> for DefaultSource
where
    T: ConfigurationBuilder,
{
    fn allows_secrets(&self) -> bool {
        true
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(T::default())
    }
}

pub(crate) mod file_source;

#[cfg(feature = "toml")]
pub(crate) mod toml_source;

#[cfg(feature = "json")]
pub(crate) mod json_source;

#[cfg(feature = "env")]
pub(crate) mod env_source;
