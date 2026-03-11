#[cfg(all(feature = "secrecy", feature = "toml"))]
mod secrecy {
    use confik::{Configuration, TomlSource};
    use indoc::indoc;
    use secrecy::{ExposeSecret as _, SecretString};

    #[test]
    fn secret_string() {
        #[derive(Debug, Configuration)]
        struct Config {
            #[confik(secret)]
            secret_string: SecretString,
        }

        let toml = indoc! {r#"
            secret_string = "SeriouslySecret"
        "#};

        let config = Config::builder()
            .override_with(TomlSource::new(toml).allow_secrets())
            .try_build()
            .unwrap();

        assert_eq!(
            format!("{:?}", config.secret_string),
            "SecretBox<str>([REDACTED])",
        );
        assert_eq!(
            format!("{config:?}"),
            "Config { secret_string: SecretBox<str>([REDACTED]) }",
        );
        assert_eq!(config.secret_string.expose_secret(), "SeriouslySecret");
    }

    #[test]
    fn secret_string_in_field_not_marked_secret() {
        #[derive(Debug, Configuration)]
        struct Config {
            secret_string: SecretString,
        }

        let toml = indoc! {r#"
            secret_string = "SeriouslySecret"
        "#};

        // in Source without `.allow_secrets()`
        Config::builder()
            .override_with(TomlSource::new(toml))
            .try_build()
            .unwrap_err();

        // in Source with `.allow_secrets()`
        let config = Config::builder()
            .override_with(TomlSource::new(toml).allow_secrets())
            .try_build()
            .unwrap();

        assert_eq!(
            format!("{:?}", config.secret_string),
            "SecretBox<str>([REDACTED])",
        );
        assert_eq!(
            format!("{config:?}"),
            "Config { secret_string: SecretBox<str>([REDACTED]) }",
        );
        assert_eq!(config.secret_string.expose_secret(), "SeriouslySecret");
    }
}

#[cfg(all(feature = "bigdecimal", feature = "toml"))]
mod bigdecimal {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use confik::{Configuration, Error, TomlSource};
    use indoc::formatdoc;

    #[derive(Configuration, Debug)]
    struct Config {
        big_decimal: BigDecimal,
    }

    #[test]
    fn bigdecimal() {
        let big_decimal = "1.414213562373095048801688724209698078569671875376948073176679737990732478462107038850387534327641573";
        let toml = formatdoc! {r#"
            big_decimal = "{big_decimal}"
        "#};

        let config = Config::builder()
            .override_with(TomlSource::new(toml))
            .try_build()
            .expect("Failed to parse config");

        assert_eq!(
            config.big_decimal,
            BigDecimal::from_str(big_decimal).unwrap()
        );
    }

    #[test]
    fn bigdecimal_missing_err_propagation() {
        let toml = formatdoc! {r#"
            big_decimal = ""
        "#};

        let config_parsing_err = Config::builder()
            .override_with(TomlSource::new(toml))
            .try_build();
        match config_parsing_err {
            Ok(_) => {
                panic!("Expected parsing error");
            }
            Err(err) => match err {
                Error::Source(source_err, _config) => {
                    assert!(source_err
                        .to_string()
                        .contains("Failed to parse empty string"));
                    assert!(source_err.to_string().contains("big_decimal"));
                }

                _ => {
                    panic!("Expected MissingValue error");
                }
            },
        }
    }
}

#[cfg(feature = "serde_json")]
mod serde_json {
    use confik::ConfigurationBuilder as _;
    use serde_json::{json, Value};

    #[test]
    fn merge_left_type_wins_over_right_type() {
        // When the two sides have different types, the left always wins
        assert_eq!(Value::Null.merge(json!({"key": "value"})), Value::Null);
        assert_eq!(json!(true).merge(json!([1, 2, 3])), json!(true));
        assert_eq!(json!(42).merge(json!({"key": "value"})), json!(42));
        assert_eq!(json!("hello").merge(json!(99)), json!("hello"));
        assert_eq!(json!([1, 2]).merge(json!({"key": "value"})), json!([1, 2]));
        assert_eq!(
            json!({"key": "value"}).merge(json!([1, 2])),
            json!({"key": "value"})
        );
    }

    #[test]
    fn merge_arrays_are_concatenated() {
        assert_eq!(json!([1, 2]).merge(json!([3, 4])), json!([1, 2, 3, 4]));
    }

    #[test]
    fn merge_objects_combine_disjoint_keys() {
        assert_eq!(
            json!({"a": 1}).merge(json!({"b": 2})),
            json!({"a": 1, "b": 2})
        );
    }

    #[test]
    fn try_build_returns_value_unchanged() {
        let value = json!({"key": "value", "num": 42});
        assert_eq!(value.clone().try_build().unwrap(), value);
    }

    #[test]
    fn contains_non_secret_data() {
        for value in [Value::Null, json!(""), json!([]), json!({})] {
            assert!(!value.contains_non_secret_data().unwrap());
        }
        for value in [
            json!(true),
            json!(42),
            json!("hello"),
            json!([1, 2, 3]),
            json!({"key": "value"}),
        ] {
            assert!(value.contains_non_secret_data().unwrap());
        }
    }

    #[cfg(feature = "toml")]
    mod toml {
        use confik::{Configuration, TomlSource};
        use serde_json::{json, Value};

        #[derive(Configuration)]
        struct Config {
            data: Value,
        }

        #[test]
        fn value_loads_from_toml() {
            let toml = r#"
                [data]
                key = "hello"
                num = 42
            "#;

            let config = Config::builder()
                .override_with(TomlSource::new(toml))
                .try_build()
                .unwrap();

            assert_eq!(config.data, json!({"key": "hello", "num": 42}));
        }

        #[test]
        fn objects_merged_from_multiple_sources() {
            let toml_1 = r#"
                [data]
                item_1 = 1
            "#;
            let toml_2 = r#"
                [data]
                item_2 = 2
            "#;

            let config = Config::builder()
                .override_with(TomlSource::new(toml_1))
                .override_with(TomlSource::new(toml_2))
                .try_build()
                .unwrap();

            assert_eq!(config.data, json!({"item_1": 1, "item_2": 2}));
        }
    }
}

#[cfg(feature = "js_option")]
mod js_option {
    use confik::Configuration;
    use js_option::JsOption;

    #[derive(Configuration, Debug)]
    struct Config {
        opt: JsOption<usize>,
    }

    #[test]
    fn undefined() {
        let config = Config::builder()
            .try_build()
            .expect("Should be valid without config");
        assert_eq!(config.opt, JsOption::Undefined);
    }

    #[cfg(feature = "json")]
    #[test]
    fn null() {
        let json = r#"{ "opt": null }"#;

        let config = Config::builder()
            .override_with(confik::JsonSource::new(json))
            .try_build()
            .expect("Failed to parse config");
        assert_eq!(config.opt, JsOption::Null);
    }

    #[cfg(feature = "toml")]
    #[test]
    fn present() {
        let toml = "opt = 5";

        let config = Config::builder()
            .override_with(confik::TomlSource::new(toml))
            .try_build()
            .expect("Should be valid without config");
        assert_eq!(config.opt, JsOption::Some(5));
    }

    #[cfg(feature = "json")]
    #[test]
    fn merge() {
        #[derive(Debug, Configuration, PartialEq, Eq)]
        struct Config {
            one: JsOption<usize>,
            two: JsOption<usize>,
            three: JsOption<usize>,
            four: JsOption<usize>,
        }

        let base = r#"{ "two": null, "three": 5 }"#;
        let merge = r#"{ "one": 1, "two": 2, "three": 3}"#;

        let config = Config::builder()
            .override_with(confik::JsonSource::new(merge))
            .override_with(confik::JsonSource::new(base))
            .try_build()
            .expect("Failed to parse config");

        assert_eq!(
            config,
            Config {
                one: JsOption::Some(1),
                two: JsOption::Null,
                three: JsOption::Some(5),
                four: JsOption::Undefined,
            }
        );
    }
}
