#[derive(confik::Configuration)]
struct Config {
    #[confik(secret = "foo")]
    _param: String,
}

fn main() {}
