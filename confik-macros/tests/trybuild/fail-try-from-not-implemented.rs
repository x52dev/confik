//! Check the error returned when `TryFrom`/`TryInto` is not implemented for the given types.

use confik::Configuration;

#[derive(Debug, Configuration)]
struct Config {
    #[confik(try_from = String)]
    param: Foo,
}

#[derive(Debug, Default, serde::Deserialize, confik::Configuration)]
struct Foo(String);

fn main() {}
