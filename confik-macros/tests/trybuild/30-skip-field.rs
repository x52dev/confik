//! Check that we can skip a field with a default

use confik::ConfigBuilder;

#[derive(Debug, Default, PartialEq, Eq)]
struct NoConfigImpl;

#[derive(confik::Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(default, skip)]
    param: NoConfigImpl,
}

fn main() {
    let config = ConfigBuilder::default()
        .try_build()
        .expect("Failed to build when configured");
    assert_eq!(
        Config {
            param: NoConfigImpl
        },
        config
    );
}
