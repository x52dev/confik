#[derive(confik::Configuration)]
struct Config {
    #[confik(default = "")]
    _param: String,
}

fn main() {}
