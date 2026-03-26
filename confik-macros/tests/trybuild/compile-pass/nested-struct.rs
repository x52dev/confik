//! Check nested structs.

use confik::Configuration;

#[derive(Debug, serde::Deserialize, Configuration, PartialEq)]
enum BasicEnum {
    A,
    B,
}

#[derive(Configuration, Debug, PartialEq)]
struct ConfigLeaf {
    param1: BasicEnum,
    param2: BasicEnum,
}

#[derive(Configuration, Debug, PartialEq)]
struct ConfigNode {
    leaf1: ConfigLeaf,
    leaf2: ConfigLeaf,
}

#[derive(Configuration, Debug, PartialEq)]
struct ConfigRoot {
    node1: ConfigNode,
    node2: ConfigNode,
}

fn main() {}
