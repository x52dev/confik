#[cfg(feature = "toml")]
mod toml {
    use confik::{Configuration, SecretOption, TomlSource};
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Eq, Deserialize)]
    #[serde(transparent)]
    struct SecretString(String);

    impl Configuration for SecretString {
        type Builder = SecretOption<Self>;
    }

    #[derive(Debug, PartialEq, Eq, Configuration)]
    struct Config {
        data: SecretString,
    }

    #[test]
    fn secrets_are_secret() {
        let toml = r#"data = "Hello World""#;

        Config::builder()
            .override_with(TomlSource::new(toml))
            .try_build()
            .expect_err("Source does not allow secrets");
    }

    #[test]
    fn secrets_sources_allow_secrets() {
        let toml = r#"data = "Hello World""#;

        let config = Config::builder()
            .override_with(TomlSource::new(toml).allow_secrets())
            .try_build()
            .expect("Secret sources allow secrets");

        assert_eq!(
            config,
            Config {
                data: SecretString("Hello World".to_string())
            }
        );
    }
}
