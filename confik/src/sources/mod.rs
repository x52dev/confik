use std::{error::Error, fmt::Debug};

use crate::ConfigurationBuilder;

/// A source of configuration data.
pub trait Source<T>: Debug {
    /// Whether this source is allowed to contain secret data.
    ///
    /// Implementations should be conservative and return `false` by default, allowing users to
    /// opt-in to storing secrets in this source.
    fn allows_secrets(&self) -> bool {
        false
    }

    /// Attempts to provide a partial configuration object from this source.
    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>>;
}

#[derive(Debug)]
pub(crate) struct DefaultSource;

impl<T> Source<T> for DefaultSource
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

pub(crate) mod file_source;

#[cfg(feature = "toml")]
pub(crate) mod toml_source;

#[cfg(feature = "json")]
pub(crate) mod json_source;

#[cfg(feature = "env")]
pub(crate) mod env_source;

pub(crate) mod offset_source;

#[cfg(test)]
pub mod test {
    use std::fmt;

    use crate::{ConfigurationBuilder, Source};

    #[derive(Clone)]
    pub(crate) struct TestSource<T> {
        pub(crate) data: T,
        pub(crate) allow_secrets: bool,
    }

    impl<T> fmt::Debug for TestSource<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("TestSource")
                .field("allow_secrets", &self.allow_secrets)
                .finish_non_exhaustive()
        }
    }

    impl<T> Source<T> for TestSource<T>
    where
        T: ConfigurationBuilder + Clone,
    {
        fn provide(&self) -> Result<T, Box<dyn std::error::Error + Sync + Send>> {
            Ok(self.data.clone())
        }

        fn allows_secrets(&self) -> bool {
            self.allow_secrets
        }
    }
}
