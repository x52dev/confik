//! Check that a really simple struct can be built.

use confik::{ConfigBuilder, Error, TomlSource};

#[derive(confik::Configuration, Debug, PartialEq)]
struct Config {
    #[confik(secret)]
    param: String,
}

fn main() {
    let error = ConfigBuilder::<Config>::default()
        .override_with(TomlSource::new(r#"param = "Hello World""#))
        .try_build()
        .expect_err("Can't build secret from Toml");
    assert_matches::assert_matches!(error, Error::UnexpectedSecret(path, _) if path.to_string().contains("`param`"));
}
