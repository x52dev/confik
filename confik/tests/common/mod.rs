use confik::{common::DatabaseConnectionConfig, Configuration, TomlSource};

#[test]
fn database_config() {
    let toml = r#"
database = "postgres"
username = "user"
password = "password"
path = "abc"
    "#;
    let config = DatabaseConnectionConfig::builder()
        .override_with(TomlSource::new(toml).allow_secrets())
        .try_build()
        .unwrap();
    assert_eq!(config.to_string(), "postgres://user:password@abc");
}
