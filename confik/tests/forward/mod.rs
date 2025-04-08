use confik::Configuration;

#[derive(Configuration, Debug, PartialEq, Eq)]
#[confik(forward(serde(rename_all = "UPPERCASE")))]
struct Container {
    field: usize,
}

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Inner {
    #[confik(forward(serde(rename = "outer")))]
    inner: usize,
}

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Field {
    #[confik(forward(serde(rename = "other_name")))]
    field1: usize,
    #[confik(forward(serde(flatten)))]
    field2: Inner,
}

#[derive(Debug, PartialEq, Eq, Configuration)]
enum Clothes {
    Hat,
    // Put some data in to force use of a custom builder
    Scarf(usize),
    #[confik(forward(serde(alias = "Gloves", alias = "SomethingElse")))]
    Other,
}

#[derive(Configuration)]
#[allow(dead_code)]
struct Cupboard {
    items: Vec<Clothes>,
}

#[cfg(feature = "toml")]
mod toml {
    use confik::{Configuration, TomlSource};

    use super::{Clothes, Container, Cupboard, Field, Inner};

    #[test]
    fn container() {
        let target = Container::builder()
            .override_with(TomlSource::new("FIELD = 1"))
            .try_build()
            .expect("Failed to build");
        assert_eq!(target, Container { field: 1 });
    }

    #[test]
    fn field() {
        let target = Field::builder()
            .override_with(TomlSource::new("other_name = 1\nouter = 2"))
            .try_build()
            .expect("Failed to build");
        assert_eq!(
            target,
            Field {
                field1: 1,
                field2: Inner { inner: 2 }
            }
        );
    }

    #[test]
    fn variant() {
        let target = Cupboard::builder()
            .override_with(TomlSource::new(
                r#"items = ["Hat", "Gloves", "SomethingElse"]"#,
            ))
            .try_build()
            .expect("Failed to build");
        assert_eq!(
            target.items.as_slice(),
            [Clothes::Hat, Clothes::Other, Clothes::Other]
        );
    }
}
