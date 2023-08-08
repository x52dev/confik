//! Useful configuration types that services will likely otherwise re-implement.

use std::{fmt, str};

use secrecy::{ExposeSecret, SecretString};

use crate::{Configuration, MissingValue};

/// The database type, used to determine the connection string format
#[derive(Debug, Clone, PartialEq, Eq, Configuration)]
#[confik(forward_serde(rename_all = "lowercase"))]
enum DatabaseKind {
    Mysql,
    Postgres,
}

impl str::FromStr for DatabaseKind {
    type Err = MissingValue;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "mysql" => Ok(Self::Mysql),
            "postgres" => Ok(Self::Postgres),
            _ => Err(Self::Err::default()),
        }
    }
}

impl fmt::Display for DatabaseKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mysql => f.write_str("mysql"),
            Self::Postgres => f.write_str("postgres"),
        }
    }
}

/// Database connection configuration, with a secret `password`.
///
/// The [`Display`] impl provides the full connection string, whereas [`Debug`] is as normal, but
/// with the `password` field value replaced by `[redacted]`.
///
/// See [`SecretBuilder`](crate::SecretBuilder) for details on secrets. NOTE: The [`Debug`] hiding
/// of the field is manually implemented for this type, and is not automatically handled by
/// `#[config(secret)]`.
///
/// [`Display`]: #impl-Display-for-DatabaseConnectionConfig
/// [`Debug`]: #impl-Debug-for-DatabaseConnectionConfig
#[derive(Clone, Configuration)]
pub struct DatabaseConnectionConfig {
    database: DatabaseKind,

    username: String,

    #[confik(secret)]
    password: SecretString,

    path: String,
}

impl fmt::Debug for DatabaseConnectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseConnectionConfig")
            .field("database", &self.database)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .field("path", &self.path)
            .finish()
    }
}

impl fmt::Display for DatabaseConnectionConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}://{}:{}@{}",
            self.database,
            self.username,
            self.password.expose_secret(),
            self.path
        )
    }
}

impl str::FromStr for DatabaseConnectionConfig {
    type Err = MissingValue;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let Some((database, input)) = input.split_once("://") else {
            return Err(Self::Err::default().prepend("database"));
        };

        let database = database
            .parse()
            .map_err(|err: MissingValue| err.prepend("database".to_string()))?;

        let Some((username, input)) = input.split_once(':') else {
            return Err(Self::Err::default().prepend("username".to_string()));
        };

        let Some((password, path)) = input.split_once('@') else {
            return Err(Self::Err::default().prepend("path".to_string()));
        };

        Ok(Self {
            database,
            username: username.to_owned(),
            password: SecretString::new(password.to_owned()),
            path: path.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_connection_string() {
        let db_config = "mysql://root:foo@localhost:3307"
            .parse::<DatabaseConnectionConfig>()
            .unwrap();
        assert_eq!(db_config.database, DatabaseKind::Mysql);
        assert_eq!(db_config.username, "root");
        assert_eq!(db_config.password.expose_secret(), "foo");
        assert_eq!(db_config.path, "localhost:3307");
    }
}
