//! Check that we can reference builder fields
use confik::Configuration;

#[derive(Configuration, Debug, PartialEq)]
struct Config {
    #[confik(default)]
    param: String,
}

type Builder = <Config as Configuration>::Builder;

fn main() {
    let _ = Builder {
        param: Default::default(),
    };
}
