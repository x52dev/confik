use std::{borrow::Cow, error::Error, fmt};

use crate::{ConfigurationBuilder, Source};

/// A [`Source`] containing raw INI data.
#[derive(Clone)]
pub struct IniSource<'a> {
    contents: Cow<'a, str>,
    allow_secrets: bool,
}

impl<'a> IniSource<'a> {
    /// Creates a [`Source`] containing raw INI data.
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

impl<T: ConfigurationBuilder> Source<T> for IniSource<'_> {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(serde_ini_0_2::from_str(&self.contents)?)
    }
}

impl fmt::Debug for IniSource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IniSource")
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
    fn provides_ini_data() {
        let source = IniSource::new("value = 42\n");

        let config =
            <IniSource<'_> as Source<<TestConfig as crate::Configuration>::Builder>>::provide(
                &source,
            )
            .unwrap()
            .try_build()
            .unwrap();

        assert_eq!(config, TestConfig { value: 42 });
    }

    #[test]
    fn propagates_parse_errors() {
        let source = IniSource::new("value\n");

        let err =
            match <IniSource<'_> as Source<<TestConfig as crate::Configuration>::Builder>>::provide(
                &source,
            ) {
                Ok(_) => panic!("INI parsing should fail"),
                Err(err) => err,
            };

        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn allow_secrets_enables_secret_loading() {
        let source = IniSource::new("value = 42\n").allow_secrets();

        assert!(<IniSource<'_> as Source<
            <TestConfig as crate::Configuration>::Builder,
        >>::allows_secrets(&source));
    }
}
