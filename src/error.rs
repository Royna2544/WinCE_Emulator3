use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse JSON from {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("CE object handle 0x{0:08x} is not valid")]
    InvalidHandle(u32),
    #[error("CE registry key not found: {0}")]
    MissingRegistryKey(String),
    #[error("CE registry value not found: {key}\\{value}")]
    MissingRegistryValue { key: String, value: String },
    #[error("CE device not found: {0}")]
    MissingDevice(String),
    #[error("emulator backend error: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, Error>;
