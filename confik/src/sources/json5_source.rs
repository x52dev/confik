use std::{borrow::Cow, error::Error, fmt};

use crate::{ConfigurationBuilder, Source};

/// A [`Source`] containing raw JSON data.
#[derive(Clone)]
pub struct Json5Source<'a> {
    contents: Cow<'a, str>,
    allow_secrets: bool,
}

impl<'a> Json5Source<'a> {
    /// Creates a [`Source`] containing raw JSON data.
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

impl<'a> Source for Json5Source<'a> {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(json5::from_str(&self.contents)?)
    }
}

impl<'a> fmt::Debug for Json5Source<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Json5Source")
            .field("allow_secrets", &self.allow_secrets)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Configuration;

    #[test]
    fn defaults() {
        let source = Json5Source::new("{}");
        assert!(!source.allows_secrets());
    }

    #[test]
    fn clone() {
        let source = Json5Source::new("{}").allow_secrets();
        assert!(source.allows_secrets());
        assert!(source.clone().allow_secrets);
    }

    #[test]
    fn json5() {
        #[derive(Configuration, Debug, PartialEq)]
        struct Config {
            message: String,
            n: i32,
        }

        let config = "
            {
              // A traditional message.
              message: 'hello world',

              // A number for some reason.
              n: 42,
            }
        ";

        assert_eq!(
            Config::builder()
                .override_with(Json5Source::new(config))
                .try_build()
                .expect("Failed to build config"),
            Config {
                message: "hello world".to_string(),
                n: 42,
            },
        );
    }
}
