//! Check nested structs.

use confik::Configuration;

#[derive(Debug, serde::Deserialize, Configuration, PartialEq, Eq)]
pub enum BasicEnum {
    A,
    B,
}

#[derive(Configuration, Debug, PartialEq, Eq)]
pub struct ConfigLeaf {
    param1: BasicEnum,
    param2: BasicEnum,
}

#[derive(Configuration, Debug, PartialEq, Eq)]
pub struct ConfigNode {
    leaf1: ConfigLeaf,
    leaf2: ConfigLeaf,
}

pub(crate) struct _ConfigRoot {
    node1: ConfigLeaf,
    node2: ConfigLeaf,
}

fn main() {}
