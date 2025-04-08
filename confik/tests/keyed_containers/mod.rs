macro_rules! create_tests_for {
    ($container:ty) => {
        use confik::Configuration;

        #[derive(Debug, Configuration, PartialEq, Eq, Hash, Ord, PartialOrd)]
        #[confik(forward(derive(Hash, PartialEq, Eq, Ord, PartialOrd)))]
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

            use super::*;

            #[test]
            fn simple() {
                let target = Target::builder()
                    .override_with(TomlSource::new("[val]\nkey = { first = 0, second = 1 }"))
                    .try_build()
                    .expect("Failed to build container from simple source");

                assert_eq!(
                    target.val.iter().collect::<Vec<_>>(),
                    [(
                        &"key".to_string(),
                        &TwoVals {
                            first: 0,
                            second: 1,
                        }
                    )],
                    "Target not equal to 0, 1: {:?}",
                    target
                );
            }

            #[test]
            fn multiple_one_source() {
                let target = Target::builder()
                    .override_with(TomlSource::new(
                        "[val]\nkey1 = { first = 0, second = 1 }\nkey2 = { first = 2, second = 3 }",
                    ))
                    .try_build()
                    .expect("Failed to build container from simple source");

                let mut result = target.val.iter().collect::<Vec<_>>();
                result.sort_unstable_by_key(|(key, _)| key.clone());

                assert_eq!(
                    result,
                    [
                        (
                            &"key1".to_string(),
                            &TwoVals {
                                first: 0,
                                second: 1,
                            }
                        ),
                        (
                            &"key2".to_string(),
                            &TwoVals {
                                first: 2,
                                second: 3,
                            }
                        )
                    ],
                    "Target not equal to 0, 1: {:?}",
                    target
                );
            }

            #[test]
            fn multiple_multiple_source() {
                let target = Target::builder()
                    .override_with(TomlSource::new("[val]\nkey1 = { first = 0, second = 1 }"))
                    .override_with(TomlSource::new("[val]\nkey2 = { first = 2, second = 3 }"))
                    .try_build()
                    .expect("Failed to build container from simple source");

                let mut result = target.val.iter().collect::<Vec<_>>();
                result.sort_unstable_by_key(|(key, _)| key.clone());

                assert_eq!(
                    result,
                    [
                        (
                            &"key1".to_string(),
                            &TwoVals {
                                first: 0,
                                second: 1,
                            }
                        ),
                        (
                            &"key2".to_string(),
                            &TwoVals {
                                first: 2,
                                second: 3,
                            }
                        )
                    ],
                    "Target not equal to 0, 1: {:?}",
                    target
                );
            }
        }

        #[cfg(feature = "json")]
        #[cfg(feature = "toml")]
        mod json {
            use confik::{Configuration, JsonSource, TomlSource};

            use super::*;

            #[test]
            fn incomplete() {
                let target = Target::builder()
                    .override_with(TomlSource::new("[val]\nkey = { second = 1 }"))
                    .override_with(JsonSource::new(
                        r#"{ "val": { "key": { "first": 0, "second": null }}}"#,
                    ))
                    .try_build()
                    .expect("Should be able to build a map from multiple sources");

                assert_eq!(
                    target.val.iter().collect::<Vec<_>>(),
                    [(
                        &"key".to_string(),
                        &TwoVals {
                            first: 0,
                            second: 1,
                        }
                    )],
                    "Target not equal to 0, 1: {:?}",
                    target
                );
            }
        }
    };
}

mod hashmap {
    use std::collections::HashMap;

    create_tests_for! { HashMap<String, TwoVals> }
}

mod btreemap {
    use std::collections::BTreeMap;

    create_tests_for! { BTreeMap<String, TwoVals> }
}
