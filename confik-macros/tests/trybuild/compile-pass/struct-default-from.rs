//! `#[confik(struct_default)]` with `from`: missing data uses `Config::default().field` (already the
//! target type); present data deserializes via the intermediate type then `Into`.
use confik::{Configuration, TomlSource};

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(from = A, struct_default)]
    param: String,
}

#[derive(Debug, Default, serde::Deserialize, confik::Configuration)]
struct A(usize);

impl From<A> for String {
    fn from(_: A) -> Self {
        String::from("from deserialized intermediate")
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
    assert_eq!(c.param, "from deserialized intermediate");
}
