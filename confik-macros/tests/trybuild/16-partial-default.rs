use confik::{ConfigBuilder, Configuration, TomlSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq, Serialize)]
struct Num(usize);

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq, Serialize)]
struct PartiallyPresent {
    a: Num,
    b: Num,
}

const DEFAULT_NUM: usize = 3;

#[derive(Debug, Default, Deserialize, Configuration, PartialEq, Eq, Serialize)]
struct Present {
    #[confik(default = def(DEFAULT_NUM))]
    partial: PartiallyPresent,
}

fn def(num: usize) -> PartiallyPresent {
    PartiallyPresent {
        a: Num(num),
        b: Num(num),
    }
}

fn main() {
    let target = ConfigBuilder::<Present>::default()
        .override_with(TomlSource::new("[partial]\na = 10\n"))
        .try_build()
        .expect("Failed to build from toml");

    assert_eq!(
        target,
        Present {
            partial: PartiallyPresent {
                a: Num(3),
                b: Num(3)
            }
        }
    );
}
