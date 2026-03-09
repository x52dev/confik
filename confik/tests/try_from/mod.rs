#[cfg(feature = "toml")]
mod toml {
    use assert_matches::assert_matches;
    use confik::{ConfigBuilder, Configuration, Error, TomlSource};
    use serde::Deserialize;

    /// An intermediate type that is deserialized from config.
    #[derive(Debug, Default, Deserialize, Configuration)]
    struct Intermediate(String);

    /// The target type we want to end up with after the `TryFrom` conversion.
    #[derive(Debug, PartialEq, Eq)]
    struct Validated(String);

    #[derive(Debug)]
    struct ValidationError(String);

    impl std::fmt::Display for ValidationError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "validation failed: {}", self.0)
        }
    }

    impl std::error::Error for ValidationError {}

    impl TryFrom<Intermediate> for Validated {
        type Error = ValidationError;

        fn try_from(value: Intermediate) -> Result<Self, Self::Error> {
            if value.0 == "valid" {
                Ok(Validated(value.0))
            } else {
                Err(ValidationError(value.0))
            }
        }
    }

    #[derive(Debug, Configuration)]
    struct Config {
        #[confik(try_from = Intermediate)]
        param: Validated,
    }

    /// A config struct where the `try_from` field is nested inside another struct,
    /// so path prepending propagates the outer field name into the error.
    #[derive(Debug, Configuration)]
    struct OuterConfig {
        #[allow(dead_code)]
        inner: Config,
    }

    #[test]
    fn try_from_failure_returns_try_into_error() {
        let result = ConfigBuilder::<Config>::default()
            .override_with(TomlSource::new(r#"param = "invalid""#))
            .try_build();

        assert_matches!(result, Err(Error::TryInto(_)));
    }

    #[test]
    fn try_from_nested_failure_path_contains_field_names() {
        let result = ConfigBuilder::<OuterConfig>::default()
            .override_with(TomlSource::new(indoc::indoc! {r#"
                [inner]
                param = "invalid"
            "#}))
            .try_build();

        let Err(Error::TryInto(err)) = result else {
            panic!("expected Error::TryInto, got {result:?}");
        };

        let err = err.to_string();
        assert!(
            err.contains("inner.param"),
            "error path should contain the outer field name 'inner', got: {err}"
        );
    }

    #[test]
    fn try_from_success_builds_correctly() {
        let config = ConfigBuilder::<Config>::default()
            .override_with(TomlSource::new(r#"param = "valid""#))
            .try_build()
            .expect("valid config should build");

        assert_eq!(config.param, Validated("valid".to_string()));
    }
}
