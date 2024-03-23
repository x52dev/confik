use confik::{Configuration, TomlSource};
use indoc::indoc;
use secrecy::{ExposeSecret as _, SecretString};

#[derive(Configuration, Debug)]
struct Config {
    secret_field: SecretString,
}

fn main() {
    let toml = indoc! {r#"
        secret_field = "ProtectedSecret"
    "#};

    let config = Config::builder()
        .override_with(TomlSource::new(toml).allow_secrets())
        .try_build()
        .expect("Failed to parse config");

    assert_eq!(
        format!("{config:?}"),
        "Config { secret_field: Secret([REDACTED alloc::string::String]) }",
    );
    assert_eq!(
        format!("{:?}", config.secret_field),
        "Secret([REDACTED alloc::string::String])",
    );
    assert_eq!(config.secret_field.expose_secret(), "ProtectedSecret");
}
