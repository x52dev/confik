//! `#[confik(struct_default)]` with `try_from`: same split as `from`, using `TryInto` when data exists.
use confik::{Configuration, TomlSource};

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(try_from = A, struct_default)]
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
        Ok(String::from("from try_from intermediate"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            param: String::from("from manual Default on Config"),
        }
    }
}

fn main() {
    let c = Config::builder().try_build().unwrap();
    assert_eq!(c.param, "from manual Default on Config");

    let c = Config::builder()
        .override_with(TomlSource::new("param = 5"))
        .try_build()
        .unwrap();
    assert_eq!(c.param, "from try_from intermediate");
}
