use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config directory not found")]
    NoDirFound,
    #[error("failed to read config at {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config at {path}: {source}")]
    ParseFailed {
        path: PathBuf,
        source: toml::de::Error,
    },
}
