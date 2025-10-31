use std::collections::{BTreeSet, HashSet};

use confik::Configuration;

#[derive(Configuration)]
#[confik(forward(derive(::std::hash::Hash, std::cmp::Ord, PartialOrd, Eq, PartialEq, Clone)))]
struct Target {
    item: usize,
}

fn main() {
    let builder = <Target as Configuration>::Builder::default();
    let mut set = HashSet::new();
    set.insert(builder.clone());
    let mut set = BTreeSet::new();
    set.insert(builder);
}
