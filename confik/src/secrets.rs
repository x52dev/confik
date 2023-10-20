use std::borrow::Cow;

use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

use crate::{path::Path, Configuration, ConfigurationBuilder, Error};

/// Captures the path of a secret found in a non-secret source.
#[derive(Debug, Default, Error)]
#[error("Found secret at path `{0}`")]
pub struct UnexpectedSecret(Path);

impl UnexpectedSecret {
    /// Prepends a path segment as we return back up the call-stack.
    #[must_use]
    pub fn prepend(mut self, path_segment: impl Into<Cow<'static, str>>) -> Self {
        self.0 .0.push(path_segment.into());
        self
    }
}

/// Wrapper type for carrying secrets, auto-applied to builders when using the `#[config(secret)]`
/// attribute.
///
/// This type causes non-secret sources (see [`Source::allows_secrets`](crate::Source::allows_secrets))
/// to error when they parse a value of this type, ensuring that only sources which allow secrets
/// contain them.
///
/// This is the only source of errors for [`ConfigurationBuilder::contains_non_secret_data`].
#[derive(Debug, Default, Deserialize)]
#[serde(bound = "T: DeserializeOwned")]
pub struct SecretBuilder<T: ConfigurationBuilder>(T);

impl<T: ConfigurationBuilder> SecretBuilder<T> {
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self(self.0.merge(other.0))
    }

    pub fn try_build(self) -> Result<T::Target, Error> {
        self.0.try_build()
    }

    pub fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        // Stop at the earliest secret, so even if we contain further `SecretBuilder`s, which have
        // returned an `Err`, reset the path.
        if self.0.contains_non_secret_data().unwrap_or(true) {
            Err(UnexpectedSecret::default())
        } else {
            Ok(false)
        }
    }
}

/// Builder for trivial types that always contain secrets, regardless of the presence of
/// `#[confik(secret)]` annotations.
///
/// This cannot be used for any case where an `Option` cannot be used as a builder, and will
/// not descend into the structure.
#[derive(Debug, Deserialize, Hash, PartialEq, PartialOrd, Eq, Ord)]
#[serde(transparent)]
pub struct SecretOption<T>(Option<T>);

impl<T> Default for SecretOption<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> ConfigurationBuilder for SecretOption<T>
where
    T: serde::de::DeserializeOwned + Configuration,
{
    type Target = T;

    fn merge(self, other: Self) -> Self {
        Self(self.0.or(other.0))
    }

    fn try_build(self) -> Result<Self::Target, Error> {
        self.0
            .ok_or_else(|| Error::MissingValue(Default::default()))
    }

    /// Should not have an `Option` wrapping a secret as `<Option<T> as ConfigurationBuilder` is
    /// used for terminal types, therefore the `SecretBuilder` wrapping would be external to it.
    fn contains_non_secret_data(&self) -> Result<bool, UnexpectedSecret> {
        match self.0 {
            Some(_) => Err(UnexpectedSecret::default()),
            None => Ok(false),
        }
    }
}
