use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use confik::Configuration;

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
    use super::*;

    use confik::JsonSource;

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
