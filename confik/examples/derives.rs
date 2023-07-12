use std::{collections::HashSet, hash::Hash};

use confik::{Configuration, TomlSource};
use indoc::indoc;

#[derive(Debug, Configuration, PartialEq, Eq)]
struct Config {
    set: HashSet<Value>,
}

#[derive(Debug, Configuration, Hash, Eq, PartialEq)]
#[confik(derive(Hash, Eq, PartialEq))]
struct Value {
    inner: String,
}

fn main() {
    let toml = indoc! {r#"
        set = [{inner = "hello"}, {inner = "world"}]
    "#};

    let config = Config::builder()
        .override_with(TomlSource::new(toml))
        .try_build()
        .expect("Failed to parse config");
    assert_eq!(
        config,
        Config {
            set: HashSet::from([
                Value {
                    inner: "hello".into()
                },
                Value {
                    inner: "world".into()
                }
            ]),
        }
    );
}
