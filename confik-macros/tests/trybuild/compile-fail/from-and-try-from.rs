//! Check that the `from` attribute works
use confik::{Configuration, TomlSource};

#[derive(Debug, Configuration, PartialEq, Eq)]
struct Config {
    #[confik(try_from = A, from = B)]
    param: String,
}

#[derive(Debug, Default, serde::Deserialize, confik::Configuration)]
struct A(usize);

#[derive(Debug, Default, serde::Deserialize, confik::Configuration)]
struct B(usize);

#[derive(Debug)]
struct E;

impl std::fmt::Display for E {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl std::error::Error for E {}

impl TryFrom<A> for String {
    type Error = E;

    fn try_from(_: A) -> Result<Self, Self::Error> {
        Ok(String::from("Hello world"))
    }
}

impl From<B> for String {
    fn from(_: B) -> Self {
        String::from("Hello world")
    }
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
