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

#[derive(Debug)]
pub(crate) struct DefaultSource;

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
