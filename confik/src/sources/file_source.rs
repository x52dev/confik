use std::{error::Error, path::PathBuf};

use cfg_if::cfg_if;
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
#[non_exhaustive]
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

    #[cfg(feature = "corn-0_10")]
    #[error(transparent)]
    Corn(#[from] libcorn_0_10::error::Error),

    #[cfg(feature = "ron-0_12")]
    #[error(transparent)]
    Ron(#[from] ron_0_12::error::SpannedError),

    #[cfg(feature = "yaml_serde-0_10")]
    #[error(transparent)]
    Yaml(#[from] yaml_serde_0_10::Error),
}

/// A [`Source`] referring to a file path.
#[derive(Debug, Clone)]
pub struct FileSource {
    path: PathBuf,
    allow_secrets: bool,
}

impl FileSource {
    /// Create a [`Source`] referring to a file path,
    ///
    /// The deserialization method will be determined by the file extension.
    ///
    /// Supported extensions:
    /// - `toml`
    /// - `json`
    /// - `ron`
    /// - `corn`
    /// - `yaml`
    /// - `yml`
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            allow_secrets: false,
        }
    }

    /// Allows this source to contain secrets.
    pub fn allow_secrets(mut self) -> Self {
        self.allow_secrets = true;
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

            Some("corn") => {
                cfg_if! {
                    if #[cfg(feature = "corn-0_10")] {
                        Ok(libcorn_0_10::from_str(&contents)?)
                    } else {
                        Err(FileErrorKind::MissingFeatureForExtension("corn"))
                    }
                }
            }

            Some("ron") => {
                cfg_if! {
                    if #[cfg(feature = "ron-0_12")] {
                        Ok(ron_0_12::from_str(&contents)?)
                    } else {
                        Err(FileErrorKind::MissingFeatureForExtension("ron"))
                    }
                }
            }

            Some("yaml" | "yml") => {
                cfg_if! {
                    if #[cfg(feature = "yaml_serde-0_10")] {
                        Ok(yaml_serde_0_10::from_str(&contents)?)
                    } else {
                        Err(FileErrorKind::MissingFeatureForExtension("yaml"))
                    }
                }
            }

            _ => Err(FileErrorKind::UnknownExtension),
        }
    }
}

impl<T: ConfigurationBuilder> Source<T> for FileSource {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        self.deserialize().map_err(|err| {
            Box::new(FileError {
                path: self.path.clone(),
                kind: err,
            }) as _
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

    #[cfg(feature = "ron-0_12")]
    #[test]
    fn ron() {
        let dir = tempfile::TempDir::new().unwrap();

        let ron_path = dir.path().join("config.ron");

        fs::write(&ron_path, "(bar:42)").unwrap();
        let source = FileSource::new(&ron_path);
        let err = source.deserialize::<Option<SimpleConfig>>().unwrap_err();
        assert!(
            err.to_string().contains("Expected option"),
            "unexpected error message: {err}",
        );

        fs::write(&ron_path, "Some((foo:42))").unwrap();
        let source = FileSource::new(&ron_path);
        let config = source.deserialize::<Option<SimpleConfig>>().unwrap();
        assert_eq!(config.unwrap().foo, 42);

        dir.close().unwrap();
    }

    #[cfg(feature = "corn-0_10")]
    #[test]
    fn corn() {
        let dir = tempfile::TempDir::new().unwrap();

        let corn_path = dir.path().join("config.corn");

        fs::write(&corn_path, "{ bar = 42 }").unwrap();
        let source = FileSource::new(&corn_path);
        let err = source.deserialize::<Option<SimpleConfig>>().unwrap_err();
        assert!(
            err.to_string().contains("missing field"),
            "unexpected error message: {err}",
        );

        fs::write(&corn_path, "{ foo = 42 }").unwrap();
        let source = FileSource::new(&corn_path);
        let config = source.deserialize::<Option<SimpleConfig>>().unwrap();
        assert_eq!(config.unwrap().foo, 42);

        dir.close().unwrap();
    }

    #[cfg(feature = "yaml_serde-0_10")]
    #[test]
    fn yaml() {
        let dir = tempfile::TempDir::new().unwrap();

        let yaml_path = dir.path().join("config.yaml");

        fs::write(&yaml_path, "{}").unwrap();
        let source = FileSource::new(&yaml_path);
        let err = source.deserialize::<Option<SimpleConfig>>().unwrap_err();
        assert!(
            err.to_string().contains("missing field"),
            "unexpected error message: {err}",
        );

        fs::write(&yaml_path, "foo: 42\n").unwrap();
        let source = FileSource::new(&yaml_path);
        let config = source.deserialize::<Option<SimpleConfig>>().unwrap();
        assert_eq!(config.unwrap().foo, 42);

        let yml_path = dir.path().join("config.yml");
        fs::write(&yml_path, "foo: 43\n").unwrap();
        let source = FileSource::new(&yml_path);
        let config = source.deserialize::<Option<SimpleConfig>>().unwrap();
        assert_eq!(config.unwrap().foo, 43);

        dir.close().unwrap();
    }
}
