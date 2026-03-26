//! Simplest check for traits implemented on the builder
use confik::Configuration;

#[derive(Configuration, Debug)]
struct Config {
    _param: String,
}

fn main() {
    let _builder = Config::builder()
        .try_build()
        .expect_err("Somehow built with no data?");
}
