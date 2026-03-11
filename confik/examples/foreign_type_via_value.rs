use confik::{Configuration, TomlSource};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ForeignType {
    item_1: usize,
    item_2: String,
}

#[derive(Configuration)]
struct Wrapper(serde_json::Value);

impl TryFrom<Wrapper> for ForeignType {
    type Error = serde_json::Error;

    fn try_from(wrapper: Wrapper) -> Result<Self, Self::Error> {
        serde_json::from_value(wrapper.0)
    }
}

#[derive(Configuration)]
struct Config {
    #[confik(try_from = Wrapper)]
    foreign_type: ForeignType,
}

fn main() {
    let toml_1 = r#"
        [foreign_type]
        item_1 = 3
    "#;
    let toml_2 = r#"
        foreign_type.item_2 = "hello"
    "#;

    let config = Config::builder()
        .override_with(TomlSource::new(toml_1))
        .override_with(TomlSource::new(toml_2))
        .try_build()
        .unwrap();
    assert_eq!(config.foreign_type.item_1, 3);
    assert_eq!(config.foreign_type.item_2, "hello");
}
