//! Check that the builder is created and can be retrieved.

#[derive(confik::Configuration)]
struct Config {
    _param: String,
}

fn main() {
    let _builder = <Config as confik::Configuration>::builder();
}
