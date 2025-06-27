use std::{error::Error, fmt, marker::PhantomData};

use crate::{ConfigurationBuilder, Source};

/// A [`Source`] containing another source that can build the target at an offset determined by
/// the provided path.
///
/// ```rust
/// # #[cfg(feature = "toml")]
/// # {
/// use confik::{helpers::BuilderOf, Configuration, OffsetSource, TomlSource};
///
/// #[derive(Debug, Configuration, PartialEq, Eq)]
/// struct Config {
///     data: usize,
///     leaf: LeafConfig,
/// }
///
/// #[derive(Debug, Configuration, PartialEq, Eq)]
/// struct LeafConfig {
///     data: usize,
/// }
///
/// let root_toml = "data = 4";
/// let leaf_toml = "data = 5";
///
/// let root_source = TomlSource::new(root_toml);
/// let leaf_source = OffsetSource::new(
///     TomlSource::new(leaf_toml),
///     |b: &mut BuilderOf<Config>| &mut b.leaf,
/// );
///
/// let config = Config::builder()
///     .override_with(root_source)
///     .override_with(leaf_source)
///     .try_build()
///     .expect("Valid source");
///
/// assert_eq!(
///     config,
///     Config {
///         data: 4,
///         leaf: LeafConfig { data: 5 }
///     }
/// );
/// # }
/// ```
pub struct OffsetSource<'a, TargetBuilder, OffsetBuilder, PathFn> {
    inner_source: Box<dyn Source<OffsetBuilder> + 'a>,
    path: PathFn,
    _phantom: PhantomData<TargetBuilder>,
}

impl<'a, OffsetBuilder, PathFn, TargetBuilder>
    OffsetSource<'a, TargetBuilder, OffsetBuilder, PathFn>
where
    TargetBuilder: ConfigurationBuilder,
    OffsetBuilder: ConfigurationBuilder,
    PathFn: for<'b> Fn(&'b mut TargetBuilder) -> &'b mut OffsetBuilder,
{
    /// Creates a [`Source`] containing raw JSON data.
    pub fn new(inner_source: impl Source<OffsetBuilder> + 'a, path: PathFn) -> Self {
        Self {
            inner_source: Box::new(inner_source),
            path,
            _phantom: PhantomData,
        }
    }
}

impl<'a, OffsetBuilder, PathFn, TargetBuilder> Source<TargetBuilder>
    for OffsetSource<'a, TargetBuilder, OffsetBuilder, PathFn>
where
    TargetBuilder: ConfigurationBuilder,
    OffsetBuilder: ConfigurationBuilder,
    PathFn: for<'b> Fn(&'b mut TargetBuilder) -> &'b mut OffsetBuilder,
{
    fn allows_secrets(&self) -> bool {
        self.inner_source.allows_secrets()
    }

    fn provide(&self) -> Result<TargetBuilder, Box<dyn Error + Sync + Send>> {
        let mut builder = TargetBuilder::default();
        *(self.path)(&mut builder) = self.inner_source.provide()?;
        Ok(builder)
    }
}

impl<TargetBuilder, OffsetBuilder, PathFn> fmt::Debug
    for OffsetSource<'_, TargetBuilder, OffsetBuilder, PathFn>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OffsetSource")
            .field("inner_source", &self.inner_source)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use crate::{helpers::BuilderOf, sources::test::TestSource, Configuration, OffsetSource};

    #[derive(Debug, Configuration, PartialEq, Eq)]
    #[confik(forward(derive(Clone)))]
    struct Config {
        #[confik(default)]
        data: usize,
        leaf: LeafConfig,
    }

    #[derive(Debug, Configuration, PartialEq, Eq)]
    #[confik(forward(derive(Clone)))]
    struct LeafConfig {
        #[confik(default)]
        data: usize,
    }

    #[test]
    fn identity_offset() {
        let test_source_builder = BuilderOf::<Config> {
            data: Some(6),
            ..Default::default()
        };
        let inner = TestSource {
            data: test_source_builder,
            allow_secrets: false,
        };

        // `std::convert::identity` can't handle the lifetimes here, probably due to early binding
        // leading to assumptions in the lifetimes.
        let source = OffsetSource::new(inner, |x| x);

        let config = Config::builder()
            .override_with(source)
            .try_build()
            .expect("Valid input");

        assert_eq!(
            config,
            Config {
                data: 6,
                leaf: LeafConfig { data: 0 }
            }
        )
    }

    #[test]
    fn leaf_offset() {
        let test_source_builder = BuilderOf::<LeafConfig> { data: Some(6) };
        let inner = TestSource {
            data: test_source_builder,
            allow_secrets: false,
        };

        let source = OffsetSource::new(inner, |x: &mut BuilderOf<Config>| &mut x.leaf);

        let config = Config::builder()
            .override_with(source)
            .try_build()
            .expect("Valid input");

        assert_eq!(
            config,
            Config {
                data: 0,
                leaf: LeafConfig { data: 6 }
            }
        )
    }

    #[test]
    #[cfg(feature = "json")]
    fn data_offset_json() {
        let data_source =
            OffsetSource::new(crate::JsonSource::new("1"), |x: &mut BuilderOf<Config>| {
                &mut x.data
            });
        let leaf_source =
            OffsetSource::new(crate::JsonSource::new("2"), |x: &mut BuilderOf<Config>| {
                &mut x.leaf.data
            });

        let config = Config::builder()
            .override_with(data_source)
            .override_with(leaf_source)
            .try_build()
            .expect("Valid input");

        assert_eq!(
            config,
            Config {
                data: 1,
                leaf: LeafConfig { data: 2 }
            }
        )
    }

    #[test]
    #[cfg(feature = "env")]
    fn data_offset_env() {
        temp_env::with_var("data", Some("10"), || {
            // `std::convert::identity` can't handle the lifetimes here, probably due to early
            // binding leading to assumptions in the lifetimes.
            let data_source = OffsetSource::new(crate::EnvSource::new(), |x| x);
            let leaf_source =
                OffsetSource::new(crate::EnvSource::new(), |x: &mut BuilderOf<Config>| {
                    &mut x.leaf
                });

            let config = Config::builder()
                .override_with(data_source)
                .override_with(leaf_source)
                .try_build()
                .expect("Valid input");

            assert_eq!(
                config,
                Config {
                    data: 10,
                    leaf: LeafConfig { data: 10 }
                }
            )
        })
    }
}
