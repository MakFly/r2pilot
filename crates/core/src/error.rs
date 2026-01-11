//! Error types for r2pilot-core

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for r2pilot-core
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for r2pilot-core
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    ConfigNotFound(PathBuf),

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidConfig(String),

    /// Cloudflare API errors
    #[error("Cloudflare API error: {0}")]
    CloudflareApi(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// R2 operation errors
    #[error("R2 operation failed: {0}")]
    R2Operation(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    /// HTTP client error
    #[error("HTTP client error: {0}")]
    HttpClient(String),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// AWS SDK error
    #[error("AWS SDK error: {0}")]
    AwsSdk(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Cancelled by user
    #[error("Operation cancelled")]
    Cancelled,

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::Timeout
        } else if err.is_connect() {
            Error::Network(err.to_string())
        } else if err.is_request() {
            Error::HttpClient(err.to_string())
        } else {
            Error::Network(err.to_string())
        }
    }
}

impl From<aws_sdk_s3::Error> for Error {
    fn from(err: aws_sdk_s3::Error) -> Self {
        Error::AwsSdk(err.to_string())
    }
}

// Generic SdkError conversion for all S3 operations
impl<E> From<aws_sdk_s3::error::SdkError<E>> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: aws_sdk_s3::error::SdkError<E>) -> Self {
        Error::R2Operation(err.to_string())
    }
}

// ByteStreamError conversion
impl From<aws_sdk_s3::primitives::ByteStreamError> for Error {
    fn from(err: aws_sdk_s3::primitives::ByteStreamError) -> Self {
        Error::R2Operation(err.to_string())
    }
}
