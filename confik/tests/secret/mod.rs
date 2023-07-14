extern crate core;

use std::{
    fmt::Debug,
    hash::{BuildHasher, Hasher},
};

use confik::Configuration;
use serde::Deserialize;

#[derive(Debug, Default, PartialEq, Eq, Configuration, Deserialize)]
struct Num(usize);

#[derive(Debug, Default, PartialEq, Eq, Configuration, Deserialize)]
struct PartiallySecret {
    public: Num,
    #[confik(secret)]
    secret: Num,
}

#[derive(Debug, Default, PartialEq, Eq, Configuration, Deserialize)]
struct NotSecret {
    public: PartiallySecret,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Configuration, Deserialize)]
#[allow(unused)]
struct MaybeSecret {
    non_secret: Option<String>,
    #[confik(secret)]
    secret: Option<String>,
}

#[derive(Debug, Configuration, Deserialize)]
#[allow(unused)]
struct MaybeSecretVec {
    seq: Vec<MaybeSecret>,
}

#[derive(Debug, Configuration, Deserialize)]
#[allow(unused)]
struct MaybeSecretArray {
    seq: [MaybeSecret; 2],
}

#[cfg(feature = "json")]
mod json {
    use assert_matches::assert_matches;
    use confik::{ConfigBuilder, Error, JsonSource};

    use super::NotSecret;

    #[test]
    fn check_json_is_not_secret() {
        let target = ConfigBuilder::<NotSecret>::default()
            .override_with(JsonSource::new(r#"{"public": {"public": 1, "secret": 2}}"#))
            .try_build()
            .expect_err("JSON deserialization is not a secret source");

        assert_matches!(
            &target,
            Error::UnexpectedSecret(path, _) if path.to_string().contains("public.secret")
        );
    }
}

#[cfg(feature = "toml")]
mod toml {
    use std::{
        collections::{BTreeMap, HashMap},
        fmt::Debug,
    };

    use assert_matches::assert_matches;
    use confik::{ConfigBuilder, Configuration, TomlSource};
    use indoc::indoc;
    use serde::{de::DeserializeOwned, Deserialize};

    use super::{DeterministicHash, MaybeSecret, MaybeSecretArray, MaybeSecretVec, NotSecret};

    #[test]
    fn check_toml_is_not_secret() {
        use confik::Error;

        let target = ConfigBuilder::<NotSecret>::default()
            .override_with(TomlSource::new("[public]\npublic = 1\nsecret = 2"))
            .try_build()
            .expect_err("Toml deserialization is not a secret source");

        assert_matches!(
            &target,
            Error::UnexpectedSecret(path, _) if path.to_string().contains("`public.secret`")
        );
    }

    /// This functions and all tests using are to catch the issue in FUT-5298
    /// in which depending on ordering a non-secret `Source` may not be caught
    fn check_secret_error_seq_propagation<T>(expected_path: &str)
    where
        T: DeserializeOwned + Configuration + Debug,
    {
        use confik::Error;

        let target = ConfigBuilder::<T>::default()
            .override_with(TomlSource::new(indoc! {r#"
                    [[seq]]
                    non_secret = "non_secret"
                    [[seq]]
                    secret = "secret"
                "#}))
            .try_build()
            .expect_err("Toml deserialization is not a secret source");

        assert_matches!(
            &target,
            Error::UnexpectedSecret(path, _) if path.to_string().contains(&format!("`{}`", expected_path))
        );
    }

    #[test]
    fn check_secret_error_vec_propagation() {
        check_secret_error_seq_propagation::<MaybeSecretVec>("seq.1.secret");
    }

    #[test]
    fn check_secret_error_array_propagation() {
        check_secret_error_seq_propagation::<MaybeSecretArray>("seq.1.secret");
    }

    fn check_secret_error_map_propagation<M>()
    where
        M: for<'a> Deserialize<'a> + Configuration + Debug,
    {
        use confik::Error;

        let target = ConfigBuilder::<M>::default()
            .override_with(TomlSource::new(indoc! {r#"
                [a]
                non_secret = "non_secret"
                [b]
                secret = "secret"
            "#}))
            .try_build()
            .expect_err("Toml deserialization is not a secret source");

        assert_matches!(
            &target,
            Error::UnexpectedSecret(path, _) if path.to_string().contains("`b.secret`")
        );
    }

    #[test]
    fn check_secret_error_hashmap_propagation() {
        check_secret_error_map_propagation::<HashMap<String, MaybeSecret, DeterministicHash>>();
    }

    #[test]
    fn check_secret_error_btreemap_propagation() {
        check_secret_error_map_propagation::<BTreeMap<String, MaybeSecret>>();
    }
}

#[cfg(feature = "env")]
mod env {
    use confik::{ConfigBuilder, EnvSource};

    use super::{NotSecret, Num, PartiallySecret};

    #[test]
    fn check_env_is_secret() {
        let result = temp_env::with_vars(
            vec![("PUBLIC__PUBLIC", Some("3")), ("PUBLIC__SECRET", Some("4"))],
            || {
                ConfigBuilder::<NotSecret>::default()
                    .override_with(EnvSource::new().allow_secrets())
                    .try_build()
            },
        );
        assert_eq!(
            result.expect("Env is a secret source"),
            NotSecret {
                public: PartiallySecret {
                    public: Num(3),
                    secret: Num(4),
                }
            }
        );
    }

    #[cfg(feature = "toml")]
    mod toml {
        use confik::{ConfigBuilder, EnvSource, TomlSource};

        use super::{NotSecret, Num, PartiallySecret};

        #[test]
        fn check_partial_secret() {
            let result = temp_env::with_vars(vec![("public__secret", Some("5"))], || {
                ConfigBuilder::<NotSecret>::default()
                    .override_with(EnvSource::new().allow_secrets())
                    .override_with(TomlSource::new("[public]\npublic = 6"))
                    .try_build()
            });

            assert_eq!(
                result.expect("Env is a secret source"),
                NotSecret {
                    public: PartiallySecret {
                        public: Num(6),
                        secret: Num(5),
                    }
                }
            );
        }

        #[test]
        fn check_partial_secret_with_prefix() {
            let result =
                temp_env::with_vars(vec![("my_prefix_public__secret", Some("5"))], move || {
                    let mut config = envious::Config::new();
                    config.with_prefix("my_prefix_");

                    ConfigBuilder::<NotSecret>::default()
                        .override_with(EnvSource::new().with_config(config).allow_secrets())
                        .override_with(TomlSource::new("[public]\npublic = 6"))
                        .try_build()
                });

            assert_eq!(
                result.expect("Env is a secret source"),
                NotSecret {
                    public: PartiallySecret {
                        public: Num(6),
                        secret: Num(5),
                    }
                }
            );
        }
    }
}

/// In order to have the `HashMap` case fail FUT-5298 determinstically,
/// we beed to ensure the entires are ordered deterministically.
#[derive(Debug, Default)]
struct DeterministicHash(u64);

impl BuildHasher for DeterministicHash {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::default()
    }
}

impl Hasher for DeterministicHash {
    fn write(&mut self, bytes: &[u8]) {
        // We want `a` before `b` same as all the others would store it
        match bytes[0] {
            b'a' => self.0 = 1,
            b'b' => self.0 = 2,
            // Not sure where this comes from, but ignore it
            255 => (),
            b => unimplemented!("{}: {}", b, String::from_utf8_lossy(bytes)),
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}
