use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use confik::Configuration;

/// `#[confik(struct_default)]` reads the field from a manual [`Default`] on the config struct
/// (`from_default` is 42 here, not `usize::default()`). `#[confik(default)]` still overrides per field.
#[derive(Debug, Configuration, PartialEq, Eq)]
struct StructDefaultRoot {
    #[confik(struct_default)]
    from_default: usize,
    #[confik(default = 100_usize)]
    explicit_default: usize,
}

impl Default for StructDefaultRoot {
    fn default() -> Self {
        Self {
            // Deliberately not `usize::default()` (0), to show `struct_default` uses this impl.
            from_default: 42,
            // Ignored for `explicit_default` when missing: `#[confik(default = 100_usize)]` wins.
            explicit_default: 999,
        }
    }
}

#[test]
fn struct_default_uses_target_default_for_missing_fields() {
    let config = StructDefaultRoot::builder().try_build().unwrap();
    assert_eq!(
        config,
        StructDefaultRoot {
            from_default: 42,
            explicit_default: 100,
        }
    );
}

#[cfg(feature = "toml")]
mod struct_default_merge {
    use confik::{ConfigBuilder, TomlSource};

    use super::StructDefaultRoot;

    #[test]
    fn partial_source_respects_struct_and_field_defaults() {
        let c = ConfigBuilder::<StructDefaultRoot>::default()
            .override_with(TomlSource::new("from_default = 5"))
            .try_build()
            .unwrap();
        assert_eq!(
            c,
            StructDefaultRoot {
                from_default: 5,
                explicit_default: 100,
            }
        );

        let c = ConfigBuilder::<StructDefaultRoot>::default()
            .override_with(TomlSource::new("explicit_default = 3"))
            .try_build()
            .unwrap();
        assert_eq!(
            c,
            StructDefaultRoot {
                from_default: 42,
                explicit_default: 3,
            }
        );
    }
}

#[derive(Debug, Configuration, PartialEq, Eq, Default)]
struct Config {
    #[confik(default = 0)]
    option: Option<usize>,
    #[confik(default = [1])]
    vec: Vec<usize>,
    #[confik(default = [2])]
    hashset: HashSet<usize>,
    #[confik(default = [3])]
    btreeset: BTreeSet<usize>,
    #[confik(default = [(4, 5)])]
    btreemap: BTreeMap<usize, usize>,
    #[confik(default = [(6, 7)])]
    hashmap: HashMap<usize, usize>,
    #[confik(default = [8])]
    array: [usize; 1],
}

#[test]
fn containers_can_default() {
    let config = Config::builder().try_build().unwrap();
    assert_eq!(
        config,
        Config {
            option: Some(0),
            vec: vec![1],
            hashset: [2].into(),
            btreeset: [3].into(),
            btreemap: [(4, 5)].into(),
            hashmap: [(6, 7)].into(),
            array: [8]
        }
    );
}

#[cfg(feature = "json")]
mod explicit_config {
    use confik::JsonSource;

    use super::*;

    #[test]
    fn containers_ignore_default_with_explicit_empty() {
        // Array can't be empty
        let json = r#"{
            "option": null,
            "vec": [],
            "hashset": [],
            "btreeset": [],
            "btreemap": {},
            "hashmap": {},
            "array": [0]
        }"#;

        // Note, `#[derive(Default)]` doesn't use `confik`'s defaults.
        let config = Config::builder()
            .override_with(JsonSource::new(json))
            .try_build()
            .unwrap();
        assert_eq!(config, Config::default());
    }
}
