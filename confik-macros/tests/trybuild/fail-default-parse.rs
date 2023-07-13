#[derive(confik::Configuration)]
struct Config {
    #[confik(default = 1 + 2)]
    _param: usize,
}

fn main() {}
