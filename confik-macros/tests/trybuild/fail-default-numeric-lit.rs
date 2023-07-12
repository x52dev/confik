#[derive(confik::Configuration)]
struct Config {
    #[confik(default = 123)]
    _param: String,
}

fn main() {}
