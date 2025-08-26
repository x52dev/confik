use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Formatter},
};

use crate::{ConfigurationBuilder, Source};

/// A [`Source`] containing raw TOML data.
#[derive(Clone)]
pub struct TomlSource<'a> {
    contents: Cow<'a, str>,
    allow_secrets: bool,
}

impl<'a> TomlSource<'a> {
    /// A [`Source`] containing raw TOML data.
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

impl<T: ConfigurationBuilder> Source<T> for TomlSource<'_> {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(toml::from_str(&self.contents)?)
    }
}

impl Debug for TomlSource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TomlSource")
            .field("allow_secrets", &self.allow_secrets)
            .finish_non_exhaustive()
    }
}
