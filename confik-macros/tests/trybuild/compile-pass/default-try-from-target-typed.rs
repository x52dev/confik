//! Same as `36-default-from-target-typed.rs` for `try_from`.
use confik::{Configuration, TomlSource};

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(try_from = A, default = String::from("when absent"))]
    param: String,
}

#[derive(Debug, Default, serde::Deserialize, confik::Configuration)]
struct A(usize);

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
        Ok(String::from("when present"))
    }
}

fn main() {
    let c = Config::builder().try_build().unwrap();
    assert_eq!(c.param, "when absent");

    let c = Config::builder()
        .override_with(TomlSource::new("param = 1"))
        .try_build()
        .unwrap();
    assert_eq!(c.param, "when present");
}
