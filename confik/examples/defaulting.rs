use confik::Configuration;

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Data {
    #[confik(default = default_elements())]
    elements: Vec<usize>,
}

fn default_elements() -> Vec<usize> {
    vec![4, 3, 2, 1]
}

const FIELD2_DEFAULT: &str = "Hello World";

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(default = 5_usize)]
    field1: usize,
    #[confik(default = FIELD2_DEFAULT)]
    field2: String,
    data: Data,
}

fn main() {
    let config = Config::builder()
        .try_build()
        .expect("Failed to parse config");

    assert_eq!(
        config,
        Config {
            field1: 5,
            field2: String::from("Hello World"),
            data: Data {
                elements: vec![4, 3, 2, 1],
            },
        }
    );
}
