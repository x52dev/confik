use std::error::Error;

use crate::{ConfigurationBuilder, Source};

/// A [`Source`] referring to environment variables.
///
/// Uses the [envious](https://docs.rs/envious) crate for interpreting env vars.
///
/// # Examples
///
/// ```
/// use confik::{ConfigBuilder, Configuration, EnvSource};
///
/// #[derive(Configuration)]
/// struct Config {
///     port: u16,
/// }
///
/// std::env::set_var("PORT", "1234");
///
/// let config = ConfigBuilder::<Config>::default()
///     .override_with(EnvSource::new())
///     .try_build()
///     .unwrap();
///
/// assert_eq!(config.port, 1234);
/// ```
#[derive(Debug, Clone)]
pub struct EnvSource<'a> {
    config: envious::Config<'a>,
    allow_secrets: bool,
}

impl<'a> Default for EnvSource<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> EnvSource<'a> {
    /// Creates a new [`Source`] referring to environment variables.
    pub fn new() -> Self {
        Self {
            config: envious::Config::new(),
            allow_secrets: false,
        }
    }

    /// Sets the envious prefix.
    ///
    /// See [`envious::Config::with_prefix()`].
    pub fn with_prefix(mut self, prefix: &'a str) -> Self {
        self.config.with_prefix(prefix);
        self
    }

    /// Sets the envious separator.
    ///
    /// See [`envious::Config::with_separator()`].
    pub fn with_separator(mut self, separator: &'a str) -> Self {
        self.config.with_separator(separator);
        self
    }

    /// Sets the envious config.
    pub fn with_config(mut self, config: envious::Config<'a>) -> Self {
        self.config = config;
        self
    }

    /// Allows this source to contain secrets.
    pub fn allow_secrets(mut self) -> Self {
        self.allow_secrets = true;
        self
    }
}

impl<'a> Source for EnvSource<'a> {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide<T: ConfigurationBuilder>(&self) -> Option<Result<T, Box<dyn Error + Sync + Send>>> {
        let configuration = self.config.build_from_env();
        match configuration {
            Ok(configuration) => Some(Ok(configuration)),
            Err(error) => Some(Err(Box::new(error))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let source = EnvSource::default();
        assert!(!source.allows_secrets());
    }

    #[test]
    fn clone() {
        let source = EnvSource::default().allow_secrets();
        assert!(source.allows_secrets());
        assert!(source.clone().allow_secrets);
    }

    #[test]
    fn separator() {
        let mut config = envious::Config::new();
        config.with_separator("++");
        config.with_prefix("CFG--");
        let config_debug = format!("{config:?}");

        let source = EnvSource::default()
            .with_prefix("CFG--")
            .with_separator("++");
        let source_debug = format!("{source:?}");

        assert!(source_debug.contains(&config_debug));
    }
}
