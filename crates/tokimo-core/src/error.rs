use thiserror::Error;

/// Unified error type for all IM platform operations.
#[derive(Error, Debug)]
pub enum ImError {
    /// Authentication failed or token expired.
    #[error("authentication error: {message}")]
    Auth { message: String },

    /// The requested resource was not found.
    #[error("not found: {resource}")]
    NotFound { resource: String },

    /// Permission denied for the requested operation.
    #[error("permission denied: {message}")]
    PermissionDenied { message: String },

    /// Rate limit exceeded; retry after the specified duration.
    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    /// Invalid parameters supplied to the API.
    #[error("invalid parameter: {message}")]
    InvalidParam { message: String },

    /// The platform returned an API-level error.
    #[error("platform error [{code}]: {message}")]
    Platform { code: i64, message: String },

    /// Network / HTTP transport error.
    #[error("network error: {0}")]
    Network(String),

    /// Serialization or deserialization error.
    #[error("serde error: {0}")]
    Serde(String),

    /// The requested feature is not supported on this platform.
    #[error("not supported: {feature} on {platform}")]
    NotSupported { feature: String, platform: String },

    /// A catch-all for unexpected errors.
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for ImError {
    fn from(e: serde_json::Error) -> Self {
        ImError::Serde(e.to_string())
    }
}

pub type ImResult<T> = Result<T, ImError>;
