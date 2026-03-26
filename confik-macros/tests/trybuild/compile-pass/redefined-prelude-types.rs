//! Check that the builder is created and can be retrieved.
use confik::Configuration;

#[allow(dead_code)]
type Option = ();
#[allow(dead_code)]
type Some = ();
#[allow(dead_code)]
type None = ();
#[allow(dead_code)]
type Result = ();
#[allow(dead_code)]
type Box = ();

#[derive(Configuration)]
struct Config {
    _param: String,
}

fn main() {
    let _builder = Config::builder();
}
