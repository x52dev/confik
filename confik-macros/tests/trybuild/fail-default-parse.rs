#[derive(confik::Configuration)]
struct Config {
    #[confik(default = 1_usize + 2_usize)]
    _param: usize,
}

fn main() {}
