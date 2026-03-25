use std::{borrow::Cow, error::Error, fmt};

use crate::{ConfigurationBuilder, Source};

/// A [`Source`] containing raw Corn data.
#[derive(Clone)]
pub struct CornSource<'a> {
    contents: Cow<'a, str>,
    allow_secrets: bool,
}

impl<'a> CornSource<'a> {
    /// Creates a [`Source`] containing raw Corn data.
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

impl<T: ConfigurationBuilder> Source<T> for CornSource<'_> {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(libcorn_0_10::from_str(&self.contents)?)
    }
}

impl fmt::Debug for CornSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CornSource")
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
    fn provides_corn_data() {
        let source = CornSource::new("{ value = 42 }");

        let config =
            <CornSource<'_> as Source<<TestConfig as crate::Configuration>::Builder>>::provide(
                &source,
            )
            .unwrap()
            .try_build()
            .unwrap();

        assert_eq!(config, TestConfig { value: 42 });
    }

    #[test]
    fn propagates_parse_errors() {
        let source = CornSource::new("{ value = ");

        let err =
            match <CornSource<'_> as Source<<TestConfig as crate::Configuration>::Builder>>::provide(
                &source,
            ) {
                Ok(_) => panic!("Corn parsing should fail"),
                Err(err) => err,
            };

        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn allow_secrets_enables_secret_loading() {
        let source = CornSource::new("{ value = 42 }").allow_secrets();

        assert!(<CornSource<'_> as Source<
            <TestConfig as crate::Configuration>::Builder,
        >>::allows_secrets(&source));
    }
}
