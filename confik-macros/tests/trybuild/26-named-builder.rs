//! Check that we can name and reference a builder.
use confik::ConfigurationBuilder;

#[derive(confik::Configuration, Debug, PartialEq)]
#[confik(name = C)]
struct Config {
    #[confik(default)]
    param: String,
}

fn main() {
    let Config { .. } = C::default()
        .try_build()
        .expect("Default builder should succeed");
}
