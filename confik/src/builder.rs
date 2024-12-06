//! A general builder struct to allow for adding sources gradually, instead of requiring
//! producing your own list for [`build_from_sources`].
//!
//! The sources are consumed in the order they are provided, with priority given to the first
//! source. E.g., If sources `Source::File("config.toml")` and `Source::File("defaults.toml")` are
//! provided any values specified in `config.toml` take precedence over `defaults.toml`.
//!
//! A builder will generally be created by calling [`ConfigBuilder::default`], sources will be added
//! with [`ConfigBuilder::override_with`] which overrides existing source with the new source, and
//! then your configuration built with [`ConfigBuilder::try_build`].

use std::{marker::PhantomData, mem};

use confik::sources::DefaultSource;

use crate::{
    build_from_sources,
    sources::{DynSource, Source},
    Configuration, Error,
};

/// Used to accumulate ordered sources from which its `Target` is to be built.
///
/// An instance of this can be created via [`Configuration::builder`] or
/// [`ConfigBuilder::<T>::default`].
///
/// # Examples
///
/// Using [`Configuration::builder`]:
///
/// ```
/// # #[cfg(feature = "toml")]
/// # {
/// use confik::{Configuration, TomlSource};
///
/// #[derive(Debug, PartialEq, Configuration)]
/// struct MyConfigType {
///     param: String,
/// }
///
/// let config = MyConfigType::builder()
///     .override_with(TomlSource::new(r#"param = "Hello World""#))
///     .try_build()
///     .expect("Failed to build");
///
/// assert_eq!(config.param, "Hello World");
/// # }
/// ```
///
/// Using [`ConfigBuilder::<T>::default`]:
///
/// ```
/// # #[cfg(feature = "toml")]
/// # {
/// use confik::{ConfigBuilder, Configuration, TomlSource};
///
/// #[derive(Debug, PartialEq, Configuration)]
/// struct MyConfigType {
///     param: String,
/// }
///
/// let config = ConfigBuilder::<MyConfigType>::default()
///     .override_with(TomlSource::new(r#"param = "Hello World""#))
///     .try_build()
///     .expect("Failed to build");
///
/// assert_eq!(config.param, "Hello World");
/// # }
/// ```
pub struct ConfigBuilder<'a, Target: Configuration> {
    sources: Vec<Box<dyn DynSource<Target::Builder> + 'a>>,

    /// Use the generic parameter
    _phantom: PhantomData<fn() -> Target>,
}

impl<'a, Target: Configuration> ConfigBuilder<'a, Target> {
    /// Add a single [`Source`] to the list of sources.
    ///
    /// The source is added at the end of the list, overriding existing sources.
    ///
    /// ```
    /// # #[cfg(feature = "toml")]
    /// # {
    /// use confik::{Configuration, TomlSource};
    /// #[derive(Debug, PartialEq, Configuration)]
    /// struct MyConfigType {
    ///     param: String,
    /// }
    ///
    /// let config = MyConfigType::builder()
    ///     .override_with(TomlSource::new(r#"param = "Hello World""#))
    ///     .override_with(TomlSource::new(r#"param = "Hello Universe""#))
    ///     .try_build()
    ///     .expect("Failed to build");
    ///
    /// assert_eq!(config.param, "Hello Universe");
    /// # }
    /// ```
    pub fn override_with(&mut self, source: impl Source + 'a) -> &mut Self {
        self.sources.push(Box::new(source));
        self
    }

    /// Attempt to build from the provided sources.
    ///
    /// # Errors
    ///
    /// Returns an error if a required value is missing, a secret value was provided in a non-secret
    /// source, or an error is returned from a source (e.g., invalid TOML). See [`Error`] for more
    /// details.
    pub fn try_build(&mut self) -> Result<Target, Error> {
        if self.sources.is_empty() {
            build_from_sources([Box::new(DefaultSource) as Box<dyn DynSource<_>>])
        } else {
            build_from_sources(mem::take(&mut self.sources).into_iter().rev())
        }
    }
}

impl<Target: Configuration> Default for ConfigBuilder<'_, Target> {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            _phantom: PhantomData,
        }
    }
}
