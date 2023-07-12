//! Check that the derive exists and compiles

#[derive(confik::Configuration)]
struct _Config {
    _param: String,
}

fn main() {}
