use thiserror::Error;

#[derive(Debug, Error)]
pub enum PkgError {
    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("dependency `{name}` not found in registry")]
    NotFound { name: String },

    #[error("version conflict: `{name}` requires {required} but {found} is locked")]
    VersionConflict { name: String, required: String, found: String },

    #[error("registry error: {0}")]
    Registry(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("checksum mismatch for `{name}`: expected {expected}, got {actual}")]
    ChecksumMismatch { name: String, expected: String, actual: String },
}
