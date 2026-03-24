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
        Ok(ron_0_12::from_str(&self.contents)?)
    }
}

impl fmt::Debug for RonSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RonSource")
            .field("allow_secrets", &self.allow_secrets)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use confik_macros::Configuration;

    use super::*;

    #[derive(Debug, PartialEq, Eq, serde::Deserialize, Configuration)]
    struct TestConfig {
        value: usize,
    }

    #[test]
    fn provides_ron_data() {
        let source = RonSource::new("(value: Some(42))");

        let config =
            <RonSource<'_> as Source<<TestConfig as crate::Configuration>::Builder>>::provide(
                &source,
            )
            .unwrap()
            .try_build()
            .unwrap();

        assert_eq!(config, TestConfig { value: 42 });
    }

    #[test]
    fn propagates_parse_errors() {
        let source = RonSource::new("(value: nope)");

        let err =
            match <RonSource<'_> as Source<<TestConfig as crate::Configuration>::Builder>>::provide(
                &source,
            ) {
                Ok(_) => panic!("RON parsing should fail"),
                Err(err) => err,
            };

        assert!(err.to_string().contains("Expected"));
    }

    #[test]
    fn allow_secrets_enables_secret_loading() {
        let source = RonSource::new("(value: Some(42))").allow_secrets();

        assert!(<RonSource<'_> as Source<
            <TestConfig as crate::Configuration>::Builder,
        >>::allows_secrets(&source));
    }
}
