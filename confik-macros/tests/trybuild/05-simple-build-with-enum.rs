//! Check that a really simple struct can be built.

use confik::Configuration;

#[derive(Debug, Configuration, PartialEq)]
enum BasicEnum {
    A,
    B(),
    C{},
}

#[derive(Configuration, Debug, PartialEq)]
struct Config {
    param: BasicEnum,
}

fn main() {}
