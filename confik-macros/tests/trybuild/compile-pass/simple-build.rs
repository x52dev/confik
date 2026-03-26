//! Check that a really simple struct can be built.

use confik::{ConfigBuilder, TomlSource};

#[derive(confik::Configuration, Debug, PartialEq)]
struct Config {
    param: String,
}

fn main() {
    let config = ConfigBuilder::default()
        .override_with(TomlSource::new(r#"param = "Hello World""#))
        .try_build()
        .expect("Failed to build");
    assert_eq!(
        Config {
            param: "Hello World".to_string()
        },
        config
    );
}
