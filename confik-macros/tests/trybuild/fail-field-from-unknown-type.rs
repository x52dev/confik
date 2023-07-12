//! Check that the `from` attribute works
use confik::{Configuration, TomlSource};

#[derive(Debug, Configuration, PartialEq, Eq)]
struct Config {
    #[confik(from = "A")]
    param: String,
}

fn main() {
    let config = Config::builder()
        .override_with(TomlSource::new("param = 5"))
        .try_build()
        .expect("Failed to build with no required data");
    assert_eq!(
        Config {
            param: String::from("Hello world")
        },
        config
    );
}
