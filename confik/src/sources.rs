use std::{error::Error, fmt::Debug};

use crate::ConfigurationBuilder;

/// A source of configuration data.
pub trait Source: Debug {
    /// Whether this source is allowed to contain secret data.
    ///
    /// Implementations should be conservative and return `false` by default, allowing users to opt
    /// into storing secrets in this source.
    fn allows_secrets(&self) -> bool {
        false
    }

    /// Attempts to provide a partial configuration object from this source.
    fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>>;
}

pub(crate) trait DynSource<T>: Debug {
    fn allows_secrets(&self) -> bool;
    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>>;
}

impl<S, T> DynSource<T> for S
where
    S: Source,
    T: ConfigurationBuilder,
{
    fn allows_secrets(&self) -> bool {
        <S as Source>::allows_secrets(self)
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        <S as Source>::provide(self)
    }
}

#[derive(Debug)]
pub(crate) struct DefaultSource;

impl<T> DynSource<T> for DefaultSource
where
    T: ConfigurationBuilder,
{
    fn allows_secrets(&self) -> bool {
        true
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(T::default())
    }
}

pub(crate) mod file_source {
    use std::{
        borrow::Cow,
        error::Error,
        path::{Path, PathBuf},
    };

    use cfg_if::cfg_if;
    use thiserror::Error;

    use crate::{ConfigurationBuilder, Source};

    #[derive(Debug, Error)]
    #[error("Could not parse {}", .path.display())]
    struct FileError {
        path: PathBuf,
        #[source]
        kind: FileErrorKind,
    }

    #[derive(Debug, Error)]
    enum FileErrorKind {
        #[error(transparent)]
        CouldNotReadFile(#[from] std::io::Error),
        #[allow(dead_code)]
        #[error("{0} feature is not enabled")]
        MissingFeatureForExtension(&'static str),
        #[error("Unknown file extension")]
        UnknownExtension,
        #[cfg(feature = "toml")]
        #[error(transparent)]
        Toml(#[from] toml::de::Error),
        #[cfg(feature = "json")]
        #[error(transparent)]
        Json(#[from] serde_json::Error),
    }

    /// A [`Source`] referring to a file path.
    #[derive(Debug, Clone)]
    pub struct FileSource<'a> {
        path: Cow<'a, Path>,
        allow_secrets: bool,
    }

    impl<'a> FileSource<'a> {
        /// Create a [`Source`] referring to a file path,
        ///
        /// The deserialization method will be determined by the file extension.
        ///
        /// Supported extensions:
        /// - `toml`
        /// - `json`
        pub fn new(path: impl Into<Cow<'a, Path>>) -> Self {
            Self {
                path: path.into(),
                allow_secrets: false,
            }
        }

        /// Allows this source to contain secrets.
        pub fn allow_secrets(mut self) -> Self {
            self.allow_secrets = true;
            self
        }

        fn deserialize<T: ConfigurationBuilder>(&self) -> Result<T, FileErrorKind> {
            #[allow(unused_variables)]
            let contents = std::fs::read_to_string(&self.path)?;
            if let Some(ext) = self.path.extension() {
                if ext == "toml" {
                    cfg_if! {
                        if #[cfg(feature = "toml")] {
                            return Ok(toml::from_str(&contents)?);
                        } else {
                            return Err(FileErrorKind::MissingFeatureForExtension("toml"));
                        }
                    }
                }
                if ext == "json" {
                    cfg_if! {
                        if #[cfg(feature = "json")] {
                            return Ok(serde_json::from_str(&contents)?);
                        } else {
                            return Err(FileErrorKind::MissingFeatureForExtension("json"));
                        }
                    }
                }
            }
            Err(FileErrorKind::UnknownExtension)
        }
    }

    impl<'a> Source for FileSource<'a> {
        fn allows_secrets(&self) -> bool {
            self.allow_secrets
        }

        fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
            self.deserialize().map_err(|e| {
                Box::new(FileError {
                    path: self.path.clone().into_owned(),
                    kind: e,
                }) as Box<_>
            })
        }
    }
}

#[cfg(feature = "toml")]
pub(crate) mod toml_source {
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

    impl<'a> Source for TomlSource<'a> {
        fn allows_secrets(&self) -> bool {
            self.allow_secrets
        }

        fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
            Ok(toml::from_str(&self.contents)?)
        }
    }

    impl<'a> Debug for TomlSource<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TomlSource")
                .field("allow_secrets", &self.allow_secrets)
                .finish_non_exhaustive()
        }
    }
}

#[cfg(feature = "json")]
pub(crate) mod json_source {
    use std::{
        borrow::Cow,
        error::Error,
        fmt::{Debug, Formatter},
    };

    use crate::{ConfigurationBuilder, Source};

    /// A [`Source`] containing raw JSON data.
    #[derive(Clone)]
    pub struct JsonSource<'a> {
        contents: Cow<'a, str>,
        allow_secrets: bool,
    }

    impl<'a> JsonSource<'a> {
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

    impl<'a> Source for JsonSource<'a> {
        fn allows_secrets(&self) -> bool {
            self.allow_secrets
        }

        fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
            Ok(serde_json::from_str(&self.contents)?)
        }
    }

    impl<'a> Debug for JsonSource<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("JsonSource")
                .field("allow_secrets", &self.allow_secrets)
                .finish_non_exhaustive()
        }
    }
}

#[cfg(feature = "env")]
pub(crate) mod env_source {
    use std::error::Error;

    use envious::Config;

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
    ///
    /// # Secrets
    ///
    /// Secrets are allowed.
    #[derive(Debug, Clone)]
    pub struct EnvSource<'a> {
        config: Config<'a>,
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
                config: Config::new(),
                allow_secrets: false,
            }
        }

        /// Sets the envious prefix.
        ///
        /// See [`Config::with_prefix`].
        pub fn with_prefix(mut self, prefix: &'a str) -> Self {
            self.config.with_prefix(prefix);
            self
        }

        /// Sets the envious separator.
        ///
        /// See [`Config::with_separator`].
        pub fn with_separator(mut self, separator: &'a str) -> Self {
            self.config.with_separator(separator);
            self
        }

        /// Sets the envious config.
        pub fn with_config(mut self, config: Config<'a>) -> Self {
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

        fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
            Ok(self.config.build_from_env()?)
        }
    }
}
