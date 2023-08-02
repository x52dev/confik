#[cfg(feature = "secrecy")]
mod secrecy {
    use confik::{Configuration, TomlSource};
    use secrecy::{ExposeSecret, SecretString};

    #[test]
    fn secret_string() {
        #[derive(Debug, Configuration)]
        struct Config {
            #[confik(secret)]
            secret_string: SecretString,
        }

        let toml = r#"
        secret_string = "SeriouslySecret"
            "#;

        let config = Config::builder()
            .override_with(TomlSource::new(toml).allow_secrets())
            .try_build()
            .unwrap();

        assert_eq!(
            "Secret([REDACTED alloc::string::String])",
            format!("{:?}", config.secret_string)
        );
        assert_eq!("SeriouslySecret", config.secret_string.expose_secret());

        assert_eq!(
            "Config { secret_string: Secret([REDACTED alloc::string::String]) }",
            format!("{:?}", config)
        );
    }
}
