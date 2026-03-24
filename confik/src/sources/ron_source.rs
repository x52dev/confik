use std::{borrow::Cow, error::Error, fmt};

use crate::{ConfigurationBuilder, Source};

/// A [`Source`] containing raw RON data.
#[derive(Clone)]
pub struct RonSource<'a> {
    contents: Cow<'a, str>,
    allow_secrets: bool,
}

impl<'a> RonSource<'a> {
    /// Creates a [`Source`] containing raw RON data.
    pub fn new(contents: impl Into<Cow<'a, str>>) -> Self {
        Self {
            contents: contents.into(),
            allow_secrets: false,
        }
    }

    /// Allows this source to contain secrets.
    pub fn allow_secrets(mut self) -> Self {
        self.allow_secrets = true;
        self
    }
}

impl<T: ConfigurationBuilder> Source<T> for RonSource<'_> {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(ron::from_str(&self.contents)?)
    }
}

impl fmt::Debug for RonSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RonSource")
            .field("allow_secrets", &self.allow_secrets)
            .finish_non_exhaustive()
    }
}
