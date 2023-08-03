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
