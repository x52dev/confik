#[derive(confik::Configuration)]
enum Config {
    String(String),
    Num(u64),
}

fn main() {}
