use ahash::{AHashMap, AHashSet};
use confik::{Configuration, TomlSource};
use indoc::formatdoc;

#[derive(Configuration, Debug)]
struct Config {
    hashset: AHashSet<u32>,
    hashmap: AHashMap<String, u32>,
}

fn main() {
    let toml = formatdoc! {r#"
        hashset = [1, 2, 3]
        [hashmap]
        first = 10
        second = 20
    "#};

    let config = Config::builder()
        .override_with(TomlSource::new(toml))
        .try_build()
        .expect("Failed to parse config");

    assert_eq!(3, config.hashset.len());
    assert!(config.hashset.contains(&1));
    assert!(config.hashset.contains(&2));
    assert!(config.hashset.contains(&3));

    assert_eq!(2, config.hashmap.len());
    assert_eq!(Some(&10), config.hashmap.get("first"));
    assert_eq!(Some(&20), config.hashmap.get("second"));
}
