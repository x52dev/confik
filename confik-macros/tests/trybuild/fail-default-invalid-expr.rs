#[derive(confik::Configuration)]
struct Config {
    #[confik(default = Hello World)]
    _param: String,
}

fn main() {}
