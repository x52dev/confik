use std::borrow::Cow;

use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

use crate::{path::Path, ConfigurationBuilder, MissingValue};

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

    pub fn try_build(self) -> Result<T::Target, MissingValue> {
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
