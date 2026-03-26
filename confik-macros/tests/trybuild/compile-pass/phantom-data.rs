//! Check that the builder is created and can be retrieved.

use std::marker::PhantomData;

use confik::Configuration;

#[derive(Configuration)]
struct Config {
    _param: PhantomData<String>,
}

fn main() {
    let _builder = Config::builder();
}
