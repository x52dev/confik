//! Check that we can reference builder fields in a different module, when they're public, using the builder's name

pub mod config {
    use confik::Configuration;

    #[derive(Configuration, Debug, PartialEq)]
    #[confik(name = Builder)]
    pub struct Config {
        #[confik(default)]
        pub param: String,
    }
}

fn main() {
    let _ = config::Builder {
        param: Default::default(),
    };
}
