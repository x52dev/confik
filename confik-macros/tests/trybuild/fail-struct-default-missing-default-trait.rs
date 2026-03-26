//! `#[confik(struct_default)]` requires the config struct to implement [`Default`].
use confik::Configuration;

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(struct_default)]
    x: u32,
}

fn main() {
    let _ = Config::builder().try_build();
}
