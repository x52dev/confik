//! Secret fields can have defaults specified on the same inner attribute.

use confik::Configuration;

#[derive(Debug, PartialEq, Eq, Configuration)]
struct Config {
    #[confik(secret, default = "\"foo\"")]
    param: String,
}

fn main() {
    let config = Config::builder()
        .try_build()
        .expect("Failed to build secret with default");

    assert_eq!(
        Config {
            param: "foo".to_string()
        },
        config
    );
}
