#[derive(confik::Configuration)]
struct Config {
    #[confik = "foo"]
    _param: String,
}

fn main() {}
