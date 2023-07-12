use confik_macros::Configuration;

#[derive(Debug, PartialEq, Eq, Configuration)]
struct NestedTargetRoot {
    inner: NestedTargetLeaf,
}

#[derive(Debug, PartialEq, Eq, Configuration)]
struct NestedTargetLeaf {
    data: usize,
}

#[cfg(feature = "json")]
mod json {
    use confik::{ConfigBuilder, JsonSource};

    use super::{NestedTargetLeaf, NestedTargetRoot};

    #[test]
    fn check_nested_json() {
        assert_eq!(
            ConfigBuilder::<NestedTargetRoot>::default()
                .override_with(JsonSource::new(r#"{"inner": {"data": 1}}"#))
                .try_build()
                .expect("JSON deserialization should succeed"),
            NestedTargetRoot {
                inner: NestedTargetLeaf { data: 1 }
            }
        );
    }
}

#[cfg(feature = "toml")]
mod toml {
    use assert_matches::assert_matches;
    use confik::{ConfigBuilder, Error, TomlSource};

    use super::{NestedTargetLeaf, NestedTargetRoot};

    #[test]
    fn check_nested_toml() {
        assert_eq!(
            ConfigBuilder::<NestedTargetRoot>::default()
                .override_with(TomlSource::new("[inner]\ndata = 2"))
                .try_build()
                .expect("Toml deserialization should succeed"),
            NestedTargetRoot {
                inner: NestedTargetLeaf { data: 2 }
            }
        );
    }

    #[test]
    fn check_missing_path() {
        assert_matches!(
            ConfigBuilder::<NestedTargetRoot>::default()
                .override_with(TomlSource::new("[inner]"))
                .try_build()
                .expect_err("Missing data"),
            Error::MissingValue(path) if path.to_string().contains("`inner.data`")
        );
    }

    #[cfg(feature = "json")]
    mod json {
        use confik::{ConfigBuilder, JsonSource, TomlSource};

        use super::{NestedTargetLeaf, NestedTargetRoot};

        #[test]
        fn check_nested_multi_source() {
            assert_eq!(
                ConfigBuilder::<NestedTargetRoot>::default()
                    .override_with(JsonSource::new(r#"{}"#))
                    .override_with(TomlSource::new("[inner]\ndata = 2"))
                    .try_build()
                    .expect("JSON + Toml deserialization should succeed"),
                NestedTargetRoot {
                    inner: NestedTargetLeaf { data: 2 }
                }
            );
            assert_eq!(
                ConfigBuilder::<NestedTargetRoot>::default()
                    .override_with(TomlSource::new(""))
                    .override_with(JsonSource::new(r#"{"inner": {"data": 1}}"#))
                    .try_build()
                    .expect("JSON + Toml deserialization should succeed"),
                NestedTargetRoot {
                    inner: NestedTargetLeaf { data: 1 }
                }
            );
        }
    }
}
