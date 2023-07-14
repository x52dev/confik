use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use confik::Configuration;

#[derive(Debug, Configuration, PartialEq, Eq)]
struct Config {
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
            vec: vec![1],
            hashset: [2].into(),
            btreeset: [3].into(),
            btreemap: [(4, 5)].into(),
            hashmap: [(6, 7)].into(),
            array: [8]
        }
    );
}
