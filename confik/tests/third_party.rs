#[cfg(feature = "secrecy")]
mod secrecy {
    use confik::{Configuration, TomlSource};
    use indoc::indoc;
    use secrecy::{ExposeSecret, SecretString};

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
            "Secret([REDACTED alloc::string::String])",
        );
        assert_eq!(
            format!("{config:?}"),
            "Config { secret_string: Secret([REDACTED alloc::string::String]) }",
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
            "Secret([REDACTED alloc::string::String])",
        );
        assert_eq!(
            format!("{config:?}"),
            "Config { secret_string: Secret([REDACTED alloc::string::String]) }",
        );
        assert_eq!(config.secret_string.expose_secret(), "SeriouslySecret");
    }
}

#[cfg(feature = "bigdecimal")]
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
