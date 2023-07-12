//! User-facing error types.
//!
//! Although in theory [`UnexpectedSecret`] and [`MissingValue`] are also user facing, they are entirely
//! handled by the `derive` internals, so is counted as internal.

use std::error::Error as StdError;

use thiserror::Error;

use crate::{MissingValue, UnexpectedSecret};

/// Possible error values.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The value contained in the `path` was not found when attempting to build
    /// the [`Configuration`](crate::Configuration) in
    /// [`ConfigurationBuilder::try_build`](crate::ConfigurationBuilder::try_build).
    #[error(transparent)]
    MissingValue(#[from] MissingValue),

    /// A wrapper around the error from one of the sources.
    #[error("Source {1} returned an error")]
    Source(#[source] Box<dyn StdError + Send + Sync>, String),

    /// The value contained in the `path` was marked as a [`SecretBuilder`](crate::SecretBuilder) but
    /// was parsed from a [`Source`](crate::Source) that was not marked as a secret
    /// (see [`Source::allows_secrets`](crate::Source::allows_secrets)).
    #[error("Found a secret in source {1} that does not permit secrets")]
    UnexpectedSecret(#[source] UnexpectedSecret, String),
}
