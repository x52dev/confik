//! Check that a really simple default can be used

use confik::{ConfigBuilder, TomlSource};

#[derive(confik::Configuration, Debug, PartialEq)]
struct Config {
    #[confik(default = "hello world")]
    param: String,
}

fn main() {
    let config = ConfigBuilder::default()
        .override_with(TomlSource::new(r#"param = "Hello World""#))
        .try_build()
        .expect("Failed to build when configured");
    assert_eq!(
        Config {
            param: "Hello World".to_string()
        },
        config
    );

    let config = ConfigBuilder::default()
        .try_build()
        .expect("Failed to build with defaults");
    assert_eq!(
        Config {
            param: "hello world".to_string()
        },
        config
    );
}
