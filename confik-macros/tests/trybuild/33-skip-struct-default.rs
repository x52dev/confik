//! Skipped field value comes from `<Config as Default>::default()`, not from the field type's
//! `Default` (`NotConfiguration` has no `Default` impl here).
use confik::ConfigBuilder;

#[derive(Clone, Debug, PartialEq, Eq)]
struct NotConfiguration(u32);

#[derive(confik::Configuration, Debug, PartialEq, Eq)]
struct Config {
    #[confik(skip, struct_default)]
    skipped: NotConfiguration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skipped: NotConfiguration(9001),
        }
    }
}

fn main() {
    let config = ConfigBuilder::<Config>::default()
        .try_build()
        .expect("Failed to build");
    assert_eq!(
        config,
        Config {
            skipped: NotConfiguration(9001),
        }
    );
}
