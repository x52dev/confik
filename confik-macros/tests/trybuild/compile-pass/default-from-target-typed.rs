//! With `from`, `#[confik(default = ...)]` may use the **target** field type (`String` here). The
//! missing-data branch must not be forced through the intermediate type.
use confik::{Configuration, TomlSource};

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(from = A, default = String::from("when absent"))]
    param: String,
}

#[derive(Debug, Default, serde::Deserialize, confik::Configuration)]
struct A(usize);

impl From<A> for String {
    fn from(_: A) -> Self {
        String::from("when present")
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
