mod array;
#[cfg(all(feature = "common", feature = "toml"))]
mod common;
mod complex_enums;
mod defaulting_containers;
mod keyed_containers;
mod option_builder;
mod secret;
mod serde_forward;
mod singly_nested_tests;
mod third_party;
mod unkeyed_containers;

use assert_matches::assert_matches;
use confik::{ConfigBuilder, Configuration, Error};
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize, Configuration)]
enum TargetEnum {
    First,
    Second,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Configuration)]
struct Target {
    a: usize,
    b: TargetEnum,
}

#[test]
fn check_no_source_fails() {
    assert_matches!(
        ConfigBuilder::<Target>::default().try_build(),
        Err(Error::MissingValue(path)) if path.to_string().contains('a')
    );
}

#[cfg(feature = "json")]
mod json {
    use confik::{ConfigBuilder, JsonSource};

    use crate::{Target, TargetEnum};

    #[test]
    fn check_json() {
        assert_eq!(
            ConfigBuilder::<Target>::default()
                .override_with(JsonSource::new(r#"{"a": 1, "b": "First"}"#))
                .try_build()
                .expect("JSON deserialization should succeed"),
            Target {
                a: 1,
                b: TargetEnum::First,
            }
        );
    }
}

#[cfg(feature = "toml")]
mod toml {
    use std::time::Duration;

    use confik::{ConfigBuilder, TomlSource};
    use confik_macros::Configuration;

    use crate::{Target, TargetEnum};

    #[test]
    fn check_toml() {
        assert_eq!(
            ConfigBuilder::<Target>::default()
                .override_with(TomlSource::new("a = 2\nb = \"Second\""))
                .try_build()
                .expect("Toml deserialization should succeed"),
            Target {
                a: 2,
                b: TargetEnum::Second,
            }
        );
    }

    #[test]
    fn from_humantime() {
        #[derive(Debug, PartialEq, Eq, Configuration)]
        struct Config {
            #[confik(forward_serde(with = "humantime_serde"))]
            timeout: Duration,
        }

        let config = ConfigBuilder::<Config>::default()
            .override_with(TomlSource::new("timeout = \"1h 42m\""))
            .try_build()
            .unwrap();

        assert_eq!(
            config,
            Config {
                timeout: Duration::from_secs(6_120)
            }
        );
    }

    #[cfg(feature = "json")]
    mod json {
        use confik::{ConfigBuilder, JsonSource, TomlSource};

        use crate::{Target, TargetEnum};

        #[test]
        fn check_multi_source() {
            assert_eq!(
                ConfigBuilder::<Target>::default()
                    .override_with(JsonSource::new(r#"{"b": "First"}"#))
                    .override_with(TomlSource::new("a = 2"))
                    .try_build()
                    .expect("JSON + Toml deserialization should succeed"),
                Target {
                    a: 2,
                    b: TargetEnum::First,
                }
            );
        }

        #[test]
        fn check_source_order() {
            assert_eq!(
                ConfigBuilder::<Target>::default()
                    .override_with(TomlSource::new("a = 2\nb = \"Second\""))
                    .override_with(JsonSource::new(r#"{"a": 1, "b": "First"}"#))
                    .try_build()
                    .expect("JSON + Toml deserialization should succeed"),
                Target {
                    a: 1,
                    b: TargetEnum::First,
                }
            );
            assert_eq!(
                ConfigBuilder::<Target>::default()
                    .override_with(JsonSource::new(r#"{"a": 1, "b": "First"}"#))
                    .override_with(TomlSource::new("a = 2\nb = \"Second\""))
                    .try_build()
                    .expect("Toml + JSON deserialization should succeed"),
                Target {
                    a: 2,
                    b: TargetEnum::Second,
                }
            );
        }

        #[test]
        fn check_error_propagation() {
            assert!(ConfigBuilder::<Target>::default()
                .override_with(TomlSource::new("a = 2\nb = \"Second\""))
                .override_with(JsonSource::new(r#"{"a": 1, "#))
                .try_build()
                .is_err());
            assert!(ConfigBuilder::<Target>::default()
                .override_with(JsonSource::new(r#"{"a": 1, "#))
                .override_with(TomlSource::new("a = 2\nb = \"Second\""))
                .try_build()
                .is_err());
        }
    }
}
