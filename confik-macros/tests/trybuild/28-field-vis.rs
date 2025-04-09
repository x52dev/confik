//! Check that we can reference builder fields in a different module, when they're public

pub mod config {
    use confik::Configuration;

    #[derive(Configuration, Debug, PartialEq)]
    pub struct Config {
        #[confik(default)]
        pub param: String,
    }

    pub type Builder = <Config as Configuration>::Builder;
}

fn main() {
    let _ = config::Builder {
        param: Default::default(),
    };
}
