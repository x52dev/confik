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
        if let Some(ext) = self.path.extension() {
            if ext == "toml" {
                cfg_if! {
                    if #[cfg(feature = "toml")] {
                        return Ok(toml::from_str(&contents)?);
                    } else {
                        return Err(FileErrorKind::MissingFeatureForExtension("toml"));
                    }
                }
            }
            if ext == "json" {
                cfg_if! {
                    if #[cfg(feature = "json")] {
                        return Ok(serde_json::from_str(&contents)?);
                    } else {
                        return Err(FileErrorKind::MissingFeatureForExtension("json"));
                    }
                }
            }
        }
        Err(FileErrorKind::UnknownExtension)
    }
}

impl Source for FileSource {
    fn allows_secrets(&self) -> bool {
        self.allow_secrets
    }

    fn provide<T: ConfigurationBuilder>(&self) -> Result<T, Box<dyn Error + Sync + Send>> {
        self.deserialize().map_err(|err| {
            Box::new(FileError {
                path: self.path.clone(),
                kind: err,
            }) as _
        })
    }
}
