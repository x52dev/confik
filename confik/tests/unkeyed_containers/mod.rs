macro_rules! create_tests_for {
    ($container:ty) => {
        use confik::Configuration;

        #[derive(Debug, Configuration, PartialEq, Eq, Hash, Ord, PartialOrd)]
        #[confik(derive(Hash, PartialEq, Eq, Ord, PartialOrd))]
        struct TwoVals {
            first: usize,
            second: usize,
        }

        #[derive(Debug, Configuration, PartialEq, Eq)]
        struct Target {
            val: $container,
        }

        #[cfg(feature = "toml")]
        mod toml {
            use confik::{Configuration, TomlSource};

            use super::{Target, TwoVals};

            #[test]
            fn simple() {
                let target = Target::builder()
                    .override_with(TomlSource::new("val = [{first = 0, second = 1}]"))
                    .try_build()
                    .expect("Failed to build container from simple source");

                assert_eq!(
                    target.val.iter().collect::<Vec<_>>(),
                    [&TwoVals {
                        first: 0,
                        second: 1
                    }],
                    "Target not equal to 0, 1: {:?}",
                    target
                );
            }

            #[cfg(feature = "json")]
            mod json {
                use confik::{Configuration, Error, JsonSource, TomlSource};

                use super::Target;

                #[test]
                fn incomplete() {
                    let err = Target::builder()
                        .override_with(TomlSource::new("val = [{ second = 1 }]"))
                        .override_with(JsonSource::new(
                            r#"{ "val": [{ "first": 0, "second": null }]"#,
                        ))
                        .try_build()
                        .expect_err("Managed to combine different items across a list?");

                    assert_matches::assert_matches!(&err, Error::Source(_, _));
                }
            }
        }
    };
}

mod hashset {
    use std::collections::HashSet;

    create_tests_for! { HashSet<TwoVals> }
}

mod btreeset {
    use std::collections::BTreeSet;

    create_tests_for! { BTreeSet<TwoVals> }
}

mod vec {
    create_tests_for! { Vec<TwoVals> }
}
