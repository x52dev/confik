use confik::Configuration;

#[derive(Configuration, Debug, PartialEq, Eq)]
enum Target {
    Simple,
    Tuple(usize, usize),
    Field { field1: usize, field2: usize },
}

#[derive(Configuration, Debug, PartialEq, Eq)]
struct RootTarget {
    target: Target,
}

#[cfg(feature = "toml")]
mod toml {
    use assert_matches::assert_matches;
    use confik::{ConfigBuilder, Error, TomlSource};

    use super::{RootTarget, Target};

    #[test]
    fn undefined() {
        let err = ConfigBuilder::<RootTarget>::default()
            .override_with(TomlSource::new(""))
            .try_build()
            .expect_err("Somehow built with no data");
        assert_matches!(
            err,
            Error::MissingValue(path) if path.to_string().contains("`target`")
        );
    }

    #[test]
    fn simple_variant() {
        let target = ConfigBuilder::<RootTarget>::default()
            .override_with(TomlSource::new("target = \"Simple\""))
            .try_build()
            .expect("Failed to build Simple");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Simple
            }
        );
    }

    #[test]
    fn tuple_variant() {
        let target = ConfigBuilder::<RootTarget>::default()
            // I hope nobody writes their config like this...
            .override_with(TomlSource::new("target = { Tuple = { 0 = 0, 1 = 1 } }"))
            .try_build()
            .expect("Failed to build Tuple");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Tuple(0, 1)
            }
        );
    }

    #[test]
    fn field_variant() {
        let target = ConfigBuilder::<RootTarget>::default()
            .override_with(TomlSource::new(
                "target = { Field = { field1 = 1, field2 = 2 } }",
            ))
            .try_build()
            .expect("Failed to build Field");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Field {
                    field1: 1,
                    field2: 2
                }
            }
        );
    }

    #[test]
    fn field_merge() {
        let target = ConfigBuilder::<RootTarget>::default()
            .override_with(TomlSource::new("target = { Field = { field2 = 2 } }"))
            .override_with(TomlSource::new("target = { Field = { field1 = 1 } }"))
            .try_build()
            .expect("Failed to build Field");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Field {
                    field1: 1,
                    field2: 2
                }
            }
        );
    }

    #[test]
    fn mix_and_match() {
        let simple_source = TomlSource::new("target = \"Simple\"");
        let field_source = TomlSource::new("target = { Field = { field1 = 1, field2 = 2 } }");

        let target = ConfigBuilder::<RootTarget>::default()
            .override_with(field_source.clone())
            .override_with(simple_source.clone())
            .try_build()
            .expect("Failed to build from mixed source");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Simple
            }
        );

        let target = ConfigBuilder::<RootTarget>::default()
            .override_with(simple_source)
            .override_with(field_source)
            .try_build()
            .expect("Failed to build from mixed source");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Field {
                    field1: 1,
                    field2: 2
                }
            }
        );
    }
}

#[cfg(feature = "json")]
mod json {
    use confik::{ConfigBuilder, JsonSource};

    use super::{RootTarget, Target};

    /// toml parsing can't do a partial load of a tuple variant
    /// If we provide `target = { TupleVariant = { 0 = 0 } }`, the error indicates that
    /// it expects only a table of length 2 and it doesn't have an explicit null type
    /// to implement it the same way we did with json
    #[test]
    fn tuple_merge() {
        let target = ConfigBuilder::<RootTarget>::default()
            // I hope nobody writes their config like this...
            .override_with(JsonSource::new(r#"{"target": { "Tuple": [0, null] }}"#))
            .override_with(JsonSource::new(r#"{"target": { "Tuple": [null, 1] }}"#))
            .try_build()
            .expect("Failed to build TupleV");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Tuple(0, 1)
            }
        );

        let target = ConfigBuilder::<RootTarget>::default()
            // I hope nobody writes their config like this...
            .override_with(JsonSource::new(r#"{"target": { "Tuple": [null, 1] }}"#))
            .override_with(JsonSource::new(r#"{"target": { "Tuple": [0, null] }}"#))
            .try_build()
            .expect("Failed to build Tuple");
        assert_eq!(
            target,
            RootTarget {
                target: Target::Tuple(0, 1)
            }
        );
    }
}
