//! `#[confik(struct_default)]` uses `<Config as Default>::default()` for that field — here `a` is
//! 8080 from a manual `Default` impl, not `u32::default()` (0).
use confik::Configuration;

#[derive(Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(struct_default)]
    a: u32,
    #[confik(default = 7_u32)]
    b: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            a: 8080,
            b: 0,
        }
    }
}

fn main() {
    let c = Config::builder().try_build().unwrap();
    assert_eq!(c, Config { a: 8080, b: 7 });
}
