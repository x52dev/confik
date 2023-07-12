use confik::Configuration;

#[derive(Debug, Eq, PartialEq, Configuration)]
struct Config {
    param: Option<usize>,
}

#[cfg(feature = "toml")]
mod toml {
    use confik::{ConfigBuilder, Configuration, TomlSource};

    use super::Config;

    #[test]
    fn not_present() {
        let config = Config::builder()
            .override_with(TomlSource::new(""))
            .try_build()
            .expect("Config with optional field not present failed to be created");
        assert_eq!(None, config.param);
    }

    #[test]
    fn present() {
        let config = ConfigBuilder::<Config>::default()
            .override_with(TomlSource::new("param = 3"))
            .try_build()
            .expect("Explicit value");
        assert_eq!(config, Config { param: Some(3) });
    }
}

#[cfg(feature = "json")]
mod json {
    use confik::{ConfigBuilder, JsonSource};

    use super::Config;

    /// Toml doesn't have a null type
    #[test]
    fn explicit_null() {
        let config = ConfigBuilder::<Config>::default()
            .override_with(JsonSource::new("{\"param\": null}"))
            .try_build()
            .expect("Explicit null");
        assert_eq!(config, Config { param: None });
    }

    #[cfg(feature = "toml")]
    mod toml {
        use confik::{ConfigBuilder, JsonSource, TomlSource};

        use super::Config;

        /// Toml doesn't have a null type
        #[test]
        fn mixed_expecting_none() {
            let config = ConfigBuilder::<Config>::default()
                .override_with(TomlSource::new(""))
                .override_with(TomlSource::new("param = 3"))
                .override_with(JsonSource::new("{\"param\": null}"))
                .override_with(TomlSource::new(""))
                .try_build()
                .expect("Explicit null");
            assert_eq!(config, Config { param: None });
        }

        /// Toml doesn't have a null type
        #[test]
        fn mixed_expecting_some() {
            let config = ConfigBuilder::<Config>::default()
                .override_with(TomlSource::new(""))
                .override_with(JsonSource::new("{\"param\": null}"))
                .override_with(TomlSource::new("param = 3"))
                .override_with(TomlSource::new(""))
                .try_build()
                .expect("Explicit null");
            assert_eq!(config, Config { param: Some(3) });
        }
    }
}

mod complex {
    use confik::Configuration;

    #[derive(Configuration, Debug, PartialEq, Eq)]
    struct Leaf {
        param: Option<usize>,
    }

    #[derive(Configuration, Debug, PartialEq, Eq)]
    struct Root {
        inner: Option<Leaf>,
    }

    #[derive(Configuration, Debug, PartialEq, Eq)]
    struct SecretLeaf {
        param: usize,
    }

    #[derive(Configuration, Debug, PartialEq, Eq)]
    struct SecretRoot {
        #[confik(secret)]
        inner: Option<SecretLeaf>,
    }

    #[cfg(feature = "toml")]
    mod toml {
        use assert_matches::assert_matches;
        use confik::{Configuration, Error, TomlSource};

        use super::{Root, SecretRoot};

        #[test]
        fn merge_unspecified() {
            let config = Root::builder()
                .override_with(TomlSource::new(""))
                .override_with(TomlSource::new("[inner]\nparam = 3"))
                .try_build()
                .expect("Merge unspecified");
            assert_eq!(Some(3), config.inner.unwrap().param);

            let config = Root::builder()
                .override_with(TomlSource::new("[inner]\nparam = 4"))
                .override_with(TomlSource::new(""))
                .try_build()
                .expect("Merge unspecified");
            assert_eq!(Some(4), config.inner.unwrap().param);
        }

        #[test]
        fn unexpected_secret() {
            let err = SecretRoot::builder()
                .override_with(TomlSource::new("[inner]\nparam = 3"))
                .try_build()
                .expect_err("Wrongly built with unexpected secret");
            assert_matches!(
                err,
                Error::UnexpectedSecret(path, _) if path.to_string().contains("`inner`")
            );
        }

        #[cfg(feature = "json")]
        mod json {
            use confik::{Configuration, JsonSource, TomlSource};

            use super::Root;

            /// Toml doesn't have a null type
            #[test]
            fn merge_none() {
                let config = Root::builder()
                    .override_with(JsonSource::new(r#"{ "inner": { "param": null } }"#))
                    .override_with(TomlSource::new("[inner]\nparam = 3"))
                    .try_build()
                    .expect("Merge unspecified");
                assert_eq!(Some(3), config.inner.unwrap().param);

                let config = Root::builder()
                    .override_with(TomlSource::new("[inner]\nparam = 3"))
                    .override_with(JsonSource::new(r#"{ "inner": { "param": null } }"#))
                    .try_build()
                    .expect("Merge unspecified");
                assert_eq!(None, config.inner.unwrap().param);
            }
        }
    }

    #[test]
    fn unspecified_secret() {
        let config = SecretRoot::builder()
            .try_build()
            .expect("Failed to build with optional secret");
        assert_eq!(None, config.inner);
    }

    #[cfg(feature = "env")]
    mod env {
        use confik::{Configuration, EnvSource};

        use super::{SecretLeaf, SecretRoot};

        #[test]
        fn expected_secret() {
            let config = temp_env::with_var("inner__param", Some("5"), || {
                SecretRoot::builder()
                    .override_with(EnvSource::new().allow_secrets())
                    .try_build()
                    .expect("Optional secret in env is allowed")
            });
            assert_eq!(
                config,
                SecretRoot {
                    inner: Some(SecretLeaf { param: 5 })
                }
            );
        }
    }
}
