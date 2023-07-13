use confik::Configuration;

#[derive(Debug, Configuration, PartialEq, Eq)]
struct Config {
    #[confik(default = +++4_u32)]
    param: u32,
}

fn main() {}
