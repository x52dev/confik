use confik::{Configuration, TomlSource};
use indoc::indoc;

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Data {
    elements: Vec<usize>,
}

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    field1: usize,
    field2: String,
    data: Data,
}

fn main() {
    let toml = indoc! {r#"
        field1 = 5
        field2 = "Hello World"

        [data]
        elements = [1, 2, 3, 4]
    "#};

    let config = Config::builder()
        .override_with(TomlSource::new(toml))
        .try_build()
        .expect("Failed to parse config");

    assert_eq!(
        config,
        Config {
            field1: 5,
            field2: String::from("Hello World"),
            data: Data {
                elements: vec![1, 2, 3, 4],
            },
        }
    );
}
