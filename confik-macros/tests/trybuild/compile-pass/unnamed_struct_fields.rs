//! Check that a really simple struct with unnamed fields can be built.

use confik::Configuration;
use confik::{ConfigBuilder, TomlSource};

#[derive(Configuration, Debug, PartialEq)]
struct Data(String);

#[derive(Configuration, Debug, PartialEq)]
struct Config {
    param: Data,
}

fn main() {
    let config = ConfigBuilder::default()
        .override_with(TomlSource::new(r#"param = "Hello World""#))
        .try_build()
        .expect("Failed to build");
    assert_eq!(
        Config {
            param: Data("Hello World".to_string())
        },
        config
    );
}
