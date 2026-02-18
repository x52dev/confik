//! Hot-reloadable configuration support.
//!
//! This module provides [`ReloadingConfig`], which wraps a configuration value
//! and allows it to be atomically reloaded at runtime.
//!
//! # Examples
//!
//! ```rust
//! # #[cfg(feature = "toml")]
//! # {
//! use confik::{Configuration, ReloadableConfig, TomlSource};
//!
//! #[derive(Debug, Configuration)]
//! struct AppConfig {
//!     port: u16,
//!     host: String,
//! }
//!
//! impl ReloadableConfig for AppConfig {
//!     type Error = confik::Error;
//!
//!     fn build() -> Result<Self, Self::Error> {
//!         Self::builder()
//!             .override_with(TomlSource::new(r#"port = 8080
//! host = "localhost""#))
//!             .try_build()
//!     }
//! }
//!
//! // Create a reloading config (no turbofish needed!)
//! let config = AppConfig::reloading().unwrap();
//!
//! // Access the current config
//! let current = config.load();
//! assert_eq!(current.port, 8080);
//!
//! // Reload from sources
//! config.reload().unwrap();
//! # }
//! ```

use std::sync::Arc;

use arc_swap::ArcSwap;

/// Trait for invoking reload callbacks.
///
/// This trait allows both `()` (no callback) and `Fn()` types to be used
/// as the callback type in `ReloadingConfig`.
pub trait ReloadCallback {
    /// Invokes the callback, if any.
    fn invoke(&self);
}

impl ReloadCallback for () {
    fn invoke(&self) {
        // No-op for unit type
    }
}

impl<F: Fn()> ReloadCallback for F {
    fn invoke(&self) {
        self()
    }
}

/// Defines how to create a new instance of [`ReloadingConfig`].
///
/// This trait is typically implemented for configuration types that need to support
/// hot-reloading. It specifies how to build a fresh instance of the configuration
/// from its sources.
pub trait ReloadableConfig: Sized {
    /// The error type returned when building fails.
    type Error;

    /// Defines the way to build the configuration item.
    ///
    /// This method should include all the logic needed to construct the configuration
    /// from its sources, including any required validations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "toml")]
    /// # {
    /// # use confik::{Configuration, ReloadableConfig, TomlSource};
    /// # #[derive(Debug, Configuration)]
    /// # struct MyConfig { value: String }
    /// impl ReloadableConfig for MyConfig {
    ///     type Error = confik::Error;
    ///
    ///     fn build() -> Result<Self, Self::Error> {
    ///         Self::builder()
    ///             .override_with(TomlSource::new(r#"value = "test""#))
    ///             .try_build()
    ///     }
    /// }
    /// # }
    /// ```
    fn build() -> Result<Self, Self::Error>;

    /// Creates a new [`ReloadingConfig`] for this configuration type.
    ///
    /// This is a convenience method that avoids needing to specify type parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if the initial configuration build fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "toml")]
    /// # {
    /// # use confik::{Configuration, ReloadableConfig, TomlSource};
    /// # #[derive(Debug, Configuration)]
    /// # struct MyConfig { value: String }
    /// # impl ReloadableConfig for MyConfig {
    /// #     type Error = confik::Error;
    /// #     fn build() -> Result<Self, Self::Error> {
    /// #         Self::builder().override_with(TomlSource::new(r#"value = "test""#)).try_build()
    /// #     }
    /// # }
    /// // Much cleaner than ReloadingConfig::<MyConfig, _>::build()
    /// let config = MyConfig::reloading().unwrap();
    /// # }
    /// ```
    fn reloading() -> Result<ReloadingConfig<Self, ()>, Self::Error> {
        ReloadingConfig::build()
    }
}

/// An instance of config that may reload itself.
///
/// This struct wraps a configuration value and allows it to be atomically swapped
/// with a newly-loaded version. Cloning this object is cheap as it only clones
/// the underlying `Arc` pointers.
///
/// # Type Parameters
///
/// * `T` - The configuration type that implements [`ReloadableConfig`]
/// * `F` - The type of the callback invoked after successful reloads (defaults to `()`), see [`ReloadCallback`]
#[derive(Debug)]
pub struct ReloadingConfig<T, F> {
    config: Arc<ArcSwap<T>>,
    on_update: F,
}

impl<T, F> Clone for ReloadingConfig<T, F>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        ReloadingConfig {
            config: Arc::clone(&self.config),
            on_update: self.on_update.clone(),
        }
    }
}

impl<T> ReloadingConfig<T, ()>
where
    T: ReloadableConfig,
{
    /// Creates a new [`ReloadingConfig`] by building the initial configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the initial configuration build fails.
    pub fn build() -> Result<Self, <T as ReloadableConfig>::Error> {
        Ok(ReloadingConfig {
            config: Arc::new(ArcSwap::new(Arc::new(T::build()?))),
            on_update: (),
        })
    }
}

impl<T, F> ReloadingConfig<T, F> {
    /// Replaces the update callback with a new one.
    ///
    /// See [`ReloadCallback`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "toml")]
    /// # {
    /// # use confik::{Configuration, ReloadableConfig, ReloadingConfig, TomlSource};
    /// # #[derive(Debug, Configuration)]
    /// # struct MyConfig { value: String }
    /// # impl ReloadableConfig for MyConfig {
    /// #     type Error = confik::Error;
    /// #     fn build() -> Result<Self, Self::Error> {
    /// #         Self::builder().override_with(TomlSource::new(r#"value = "test""#)).try_build()
    /// #     }
    /// # }
    /// let config = ReloadingConfig::<MyConfig, _>::build().unwrap()
    ///     .with_on_update(|| println!("Config reloaded!"));
    /// # }
    /// ```
    #[must_use]
    pub fn with_on_update<U>(self, new: U) -> ReloadingConfig<T, U> {
        ReloadingConfig {
            config: self.config,
            on_update: new,
        }
    }

    /// Loads the current configuration.
    ///
    /// Returns an `Arc` to the current configuration value. This is a cheap operation
    /// that doesn't block writers.
    #[must_use]
    pub fn load(&self) -> Arc<T> {
        self.config.load_full()
    }
}

impl<T, F> ReloadingConfig<T, F>
where
    T: ReloadableConfig,
    F: ReloadCallback,
{
    /// Attempts to reload the configuration.
    ///
    /// On success, calls the stored update function (if any).
    /// On error, leaves the configuration unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error if building the new configuration fails. In this case,
    /// the current configuration remains unchanged and the update callback is
    /// not invoked.
    pub fn reload(&self) -> Result<(), <T as ReloadableConfig>::Error> {
        let config = T::build()?;
        self.config.store(Arc::new(config));
        self.on_update.invoke();
        Ok(())
    }
}

#[cfg(feature = "signal")]
impl<T, F> ReloadingConfig<T, F>
where
    T: ReloadableConfig + Send + Sync + 'static,
    F: ReloadCallback + Clone + Send + Sync + 'static,
{
    /// Sets a listener for SIGHUP.
    ///
    /// This spawns a thread and listens for a signal using the [`signal_hook`] crate,
    /// with all of that crate's caveats. If you're setting signals already, you may wish to
    /// configure this yourself.
    ///
    /// When a SIGHUP signal is received, the configuration will be reloaded. If the reload
    /// fails and the `tracing` feature is enabled, an error will be logged.
    ///
    /// # Errors
    ///
    /// Returns an error if signal registration fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(all(feature = "signal", feature = "toml"))]
    /// # {
    /// # use confik::{Configuration, ReloadableConfig, ReloadingConfig, TomlSource};
    /// # #[derive(Debug, Configuration)]
    /// # struct MyConfig { value: String }
    /// # impl ReloadableConfig for MyConfig {
    /// #     type Error = confik::Error;
    /// #     fn build() -> Result<Self, Self::Error> {
    /// #         Self::builder().override_with(TomlSource::new(r#"value = "test""#)).try_build()
    /// #     }
    /// # }
    /// let config = ReloadingConfig::<MyConfig, _>::build().unwrap();
    ///
    /// // Set up signal handler
    /// let handle = config.set_signal_handler().unwrap();
    ///
    /// // The config will now reload when receiving SIGHUP
    /// // handle.join().unwrap(); // Wait for the signal handler thread
    /// # }
    /// ```
    pub fn set_signal_handler(&self) -> Result<std::thread::JoinHandle<()>, std::io::Error>
    where
        <T as ReloadableConfig>::Error: std::fmt::Display,
    {
        use signal_hook::{consts::SIGHUP, iterator::Signals};

        let mut signals = Signals::new([SIGHUP])?;
        let config = self.clone();
        Ok(std::thread::spawn(move || {
            for signal in &mut signals {
                if signal == SIGHUP {
                    if let Err(err) = config.reload() {
                        #[cfg(feature = "tracing")]
                        tracing::error!(%err, "Failed to reload configuration");

                        #[cfg(not(feature = "tracing"))]
                        {
                            // Avoid unused variable warning
                            let _ = err;
                        }
                    }
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Configuration;

    #[derive(Debug, Clone, PartialEq, Configuration)]
    struct TestConfig {
        value: u32,
    }

    impl ReloadableConfig for TestConfig {
        type Error = &'static str;

        fn build() -> Result<Self, Self::Error> {
            Ok(TestConfig { value: 42 })
        }
    }

    #[test]
    fn test_build_and_load() {
        let config = ReloadingConfig::<TestConfig, _>::build().unwrap();
        let current = config.load();
        assert_eq!(current.value, 42);
    }

    #[test]
    fn test_reload_without_callback() {
        let config = TestConfig::reloading().unwrap();
        config.reload().unwrap();
        let current = config.load();
        assert_eq!(current.value, 42);
    }

    #[test]
    fn test_reload_with_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = Arc::clone(&called);

        let config = TestConfig::reloading().unwrap().with_on_update(move || {
            called_clone.store(true, Ordering::SeqCst);
        });

        assert!(!called.load(Ordering::SeqCst));
        config.reload().unwrap();
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_reload_updates_all_clones() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(0);

        #[derive(Debug, serde::Deserialize, Configuration)]
        struct CountingConfig {
            id: u32,
        }

        impl ReloadableConfig for CountingConfig {
            type Error = std::convert::Infallible;

            fn build() -> Result<Self, Self::Error> {
                Ok(CountingConfig {
                    id: COUNTER.fetch_add(1, Ordering::SeqCst),
                })
            }
        }

        let config1 = CountingConfig::reloading().unwrap();
        let config2 = config1.clone();

        assert_eq!(config1.load().id, 0);
        assert_eq!(config2.load().id, 0);

        config1.reload().unwrap();

        assert_eq!(config1.load().id, 1);
        assert_eq!(config2.load().id, 1);
    }

    #[test]
    fn test_reload_error_leaves_config_unchanged() {
        use std::sync::atomic::{AtomicBool, Ordering};

        static SHOULD_FAIL: AtomicBool = AtomicBool::new(false);

        #[derive(Debug, serde::Deserialize, Configuration)]
        struct FallibleConfig {
            value: u32,
        }

        impl ReloadableConfig for FallibleConfig {
            type Error = &'static str;

            fn build() -> Result<Self, Self::Error> {
                if SHOULD_FAIL.load(Ordering::SeqCst) {
                    Err("Build failed")
                } else {
                    Ok(FallibleConfig { value: 42 })
                }
            }
        }

        let config = FallibleConfig::reloading().unwrap();
        assert_eq!(config.load().value, 42);

        // Make the next build fail
        SHOULD_FAIL.store(true, Ordering::SeqCst);

        // Reload should fail and leave config unchanged
        let result = config.reload();
        assert!(result.is_err());
        assert_eq!(config.load().value, 42); // Still the old value

        // Make build succeed again
        SHOULD_FAIL.store(false, Ordering::SeqCst);
        config.reload().unwrap();
        assert_eq!(config.load().value, 42);
    }

    #[test]
    fn test_callback_not_invoked_on_reload_error() {
        use std::sync::atomic::{AtomicBool, Ordering};

        static SHOULD_FAIL: AtomicBool = AtomicBool::new(false);

        #[derive(Debug, serde::Deserialize, Configuration)]
        struct FallibleConfig {
            value: u32,
        }

        impl ReloadableConfig for FallibleConfig {
            type Error = &'static str;

            fn build() -> Result<Self, Self::Error> {
                if SHOULD_FAIL.load(Ordering::SeqCst) {
                    Err("Build failed")
                } else {
                    Ok(FallibleConfig { value: 100 })
                }
            }
        }

        let callback_called = Arc::new(AtomicBool::new(false));
        let callback_called_clone = Arc::clone(&callback_called);

        let config = FallibleConfig::reloading()
            .unwrap()
            .with_on_update(move || {
                callback_called_clone.store(true, Ordering::SeqCst);
            });

        // Initial value should be 100
        assert_eq!(config.load().value, 100);

        // Successful reload should call callback
        config.reload().unwrap();
        assert!(callback_called.load(Ordering::SeqCst));

        // Reset flag
        callback_called.store(false, Ordering::SeqCst);

        // Make next build fail
        SHOULD_FAIL.store(true, Ordering::SeqCst);

        // Failed reload should NOT call callback
        let result = config.reload();
        assert!(result.is_err());
        assert!(!callback_called.load(Ordering::SeqCst));
    }
}
