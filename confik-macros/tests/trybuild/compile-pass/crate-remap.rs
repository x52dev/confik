//! Check that we import from a re-mapped confik crate root.

use ::confik as confik_remapped;

use confik_remapped::Configuration as _;

#[derive(Debug, PartialEq, confik_remapped::Configuration)]
#[confik(crate = confik_remapped)]
struct Config {}

fn main() {
    let config = Config::builder()
        .try_build()
        .expect("Failed to build when configured");

    assert_eq!(Config {}, config);
}
