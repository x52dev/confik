use std::{error::Error, path::PathBuf};

use cfg_if::cfg_if;
use log::debug;
use thiserror::Error;

use crate::{ConfigurationBuilder, Source};

#[derive(Debug, Error)]
#[error("Could not parse {}", .path.display())]
struct FileError {
    path: PathBuf,

    #[source]
    kind: FileErrorKind,
}

#[derive(Debug, Error)]
enum FileErrorKind {
    #[error(transparent)]
    CouldNotReadFile(#[from] std::io::Error),

    #[allow(dead_code)]
    #[error("{0} feature is not enabled")]
    MissingFeatureForExtension(&'static str),

    #[error("Unknown file extension")]
    UnknownExtension,

    #[cfg(feature = "toml")]
    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[cfg(feature = "json")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// A [`Source`] referring to a file path.
#[derive(Debug, Clone)]
pub struct FileSource {
    path: PathBuf,
    allow_secrets: bool,
    can_be_optional: bool,
}

impl FileSource {
    /// Create a [`Source`] referring to a file path,
    ///
    /// The deserialization method will be determined by the file extension.
    ///
    /// Supported extensions:
    /// - `toml`
    /// - `json`
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            allow_secrets: false,
            can_be_optional: false,
        }
    }

    /// Allows this source to contain secrets.
    pub fn allow_secrets(mut self) -> Self {
        self.allow_secrets = true;
        self
    }
    /// Allows the underlying configuration file represented by this [FileSource] to be missing.
    /// It won't be considered an error if the file is not found.
    pub fn allow_missing(mut self) -> Self {
        self.can_be_optional = true;
        self
    }

    fn deserialize<T: ConfigurationBuilder>(&self) -> Result<T, FileErrorKind> {
        #[allow(unused_variables)]
        let contents = std::fs::read_to_string(&self.path)?;

        match self.path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => {
                cfg_if! {
                    if #[cfg(feature = "toml")] {
                        Ok(toml::from_str(&contents)?)
                    } else {
                        Err(FileErrorKind::MissingFeatureForExtension("toml"))
                    }
                }
            }

            Some("json") => {
                cfg_if! {
                    if #[cfg(feature = "json")] {
                        Ok(serde_json::from_str(&contents)?)
                    } else {
                        Err(FileErrorKind::MissingFeatureForExtension("json"))
                    }
                }
            }

            _ => Err(FileErrorKind::UnknownExtension),
        }
    }
}

impl Source for FileSource {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide<T: ConfigurationBuilder>(&self) -> Option<Result<T, Box<dyn Error + Sync + Send>>> {
        let deserialized = self.deserialize();

        Some(match deserialized {
            Ok(configuration) => Ok(configuration),
            Err(file_error_kind) => {
                if let FileErrorKind::CouldNotReadFile(_) = file_error_kind {
                    if self.can_be_optional {
                        // Optional resources are allowed to be missing
                        debug!("Optional file source {:?} not found. Ignoring.", self.path);
                        return None;
                    }
                }
                Err(Box::new(FileError {
                    path: self.path.clone(),
                    kind: file_error_kind,
                }) as _)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use confik_macros::Configuration;

    use super::*;

    #[derive(Debug, Default, serde::Deserialize, Configuration)]
    struct NoopConfig {}

    #[derive(Debug, Default, serde::Deserialize, Configuration)]
    #[allow(dead_code)]
    struct SimpleConfig {
        foo: u64,
    }

    #[test]
    fn defaults() {
        let source = FileSource::new("config.json");
        assert!(!source.allows_secrets());
    }

    #[test]
    fn clone() {
        let source = FileSource::new("config.toml").allow_secrets();
        assert!(source.allows_secrets());
        assert!(source.clone().allow_secrets);
    }

    #[test]
    fn non_existent() {
        let source = FileSource::new("non-existent-config.toml");
        let err = source.deserialize::<Option<NoopConfig>>().unwrap_err();
        assert!(
            err.to_string().contains("No such file or directory"),
            "unexpected error message: {err}",
        );
    }

    #[test]
    fn unknown_extension() {
        let dir = tempfile::TempDir::new().unwrap();

        let cfg_path = dir.path().join("config.cfg");
        fs::write(&cfg_path, "").unwrap();

        let source = FileSource::new(&cfg_path);
        let err = source.deserialize::<Option<NoopConfig>>().unwrap_err();
        assert!(
            err.to_string().contains("Unknown file extension"),
            "unexpected error message: {err}",
        );

        dir.close().unwrap();
    }

    #[test]
    fn allow_missing() {
        let source = FileSource::new("non-existent-config.toml").allow_missing();
        let config = source.provide::<Option<NoopConfig>>();
        assert!(config.is_none());
    }

    #[cfg(feature = "json")]
    #[test]
    fn json() {
        let dir = tempfile::TempDir::new().unwrap();

        let json_path = dir.path().join("config.json");

        fs::write(&json_path, "{}").unwrap();
        let source = FileSource::new(&json_path);
        let err = source.deserialize::<Option<SimpleConfig>>().unwrap_err();
        assert!(
            err.to_string().contains("missing field"),
            "unexpected error message: {err}",
        );

        fs::write(&json_path, "{\"foo\":42}").unwrap();
        let source = FileSource::new(&json_path);
        let config = source.deserialize::<Option<SimpleConfig>>().unwrap();
        assert_eq!(config.unwrap().foo, 42);

        dir.close().unwrap();
    }

    #[cfg(feature = "toml")]
    #[test]
    fn toml() {
        let dir = tempfile::TempDir::new().unwrap();

        let toml_path = dir.path().join("config.toml");

        fs::write(&toml_path, "").unwrap();
        let source = FileSource::new(&toml_path);
        let err = source.deserialize::<Option<SimpleConfig>>().unwrap_err();
        assert!(
            err.to_string().contains("missing field"),
            "unexpected error message: {err}",
        );

        fs::write(&toml_path, "foo = 42").unwrap();
        let source = FileSource::new(&toml_path);
        let config = source.deserialize::<Option<SimpleConfig>>().unwrap();
        assert_eq!(config.unwrap().foo, 42);

        dir.close().unwrap();
    }
}
