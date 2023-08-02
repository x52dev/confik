use confik::{Configuration, TomlSource};
use indoc::indoc;
use secrecy::{ExposeSecret, SecretString};

#[derive(Configuration, Debug)]
struct Config {
    secret_field: SecretString,
}

fn main() {
    let toml = indoc! {r#"
        secret_field = "ProtectedSecret"
    "#};

    let config = Config::builder()
        .override_with(TomlSource::new(toml))
        .try_build()
        .expect("Failed to parse config");

    assert_eq!(
        "Config { secret_field: Secret([REDACTED alloc::string::String]) }",
        format!("{:?}", config)
    );
    assert_eq!(
        "Secret([REDACTED alloc::string::String])",
        format!("{:?}", config.secret_field)
    );
    assert_eq!("ProtectedSecret", config.secret_field.expose_secret());
}
