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

    /// Multipart upload error
    #[error("Multipart upload error: {0}")]
    MultipartUpload(String),

    /// Presigned URL configuration error
    #[error("Invalid presigned URL configuration: {0}")]
    PresignedUrlConfig(String),

    /// CORS configuration error
    #[error("CORS configuration error: {0}")]
    CorsConfig(String),

    /// Lifecycle rule error
    #[error("Lifecycle rule error: {0}")]
    LifecycleRule(String),

    /// Bucket settings error
    #[error("Bucket settings error: {0}")]
    BucketSettings(String),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_config() {
        let error = Error::Config("Test config error".to_string());
        assert!(error.to_string().contains("Configuration error"));
        assert!(error.to_string().contains("Test config error"));
    }

    #[test]
    fn test_error_config_not_found() {
        let error = Error::ConfigNotFound(std::path::PathBuf::from("config.toml"));
        assert!(error.to_string().contains("Configuration file not found"));
        assert!(error.to_string().contains("config.toml"));
    }

    #[test]
    fn test_error_invalid_config() {
        let error = Error::InvalidConfig("Invalid TOML format".to_string());
        assert!(error.to_string().contains("Invalid configuration format"));
        assert!(error.to_string().contains("Invalid TOML format"));
    }

    #[test]
    fn test_error_cloudflare_api() {
        let error = Error::CloudflareApi("API rate limit exceeded".to_string());
        assert!(error.to_string().contains("Cloudflare API error"));
        assert!(error.to_string().contains("API rate limit exceeded"));
    }

    #[test]
    fn test_error_authentication() {
        let error = Error::Authentication("Invalid credentials".to_string());
        assert!(error.to_string().contains("Authentication failed"));
        assert!(error.to_string().contains("Invalid credentials"));
    }

    #[test]
    fn test_error_r2_operation() {
        let error = Error::R2Operation("Bucket not found".to_string());
        assert!(error.to_string().contains("R2 operation failed"));
        assert!(error.to_string().contains("Bucket not found"));
    }

    #[test]
    fn test_error_network() {
        let error = Error::Network("Connection timeout".to_string());
        assert!(error.to_string().contains("Network error"));
        assert!(error.to_string().contains("Connection timeout"));
    }

    #[test]
    fn test_error_http_client() {
        let error = Error::HttpClient("400 Bad Request".to_string());
        assert!(error.to_string().contains("HTTP client error"));
        assert!(error.to_string().contains("400 Bad Request"));
    }

    #[test]
    fn test_error_not_found() {
        let error = Error::NotFound("Resource not found".to_string());
        assert!(error.to_string().contains("Not found"));
        assert!(error.to_string().contains("Resource not found"));
    }

    #[test]
    fn test_error_invalid_input() {
        let error = Error::InvalidInput("Invalid email format".to_string());
        assert!(error.to_string().contains("Invalid input"));
        assert!(error.to_string().contains("Invalid email format"));
    }

    #[test]
    fn test_error_permission_denied() {
        let error = Error::PermissionDenied("Access denied".to_string());
        assert!(error.to_string().contains("Permission denied"));
        assert!(error.to_string().contains("Access denied"));
    }

    #[test]
    fn test_error_timeout() {
        let error = Error::Timeout;
        assert!(error.to_string().contains("Operation timed out"));
    }

    #[test]
    fn test_error_cancelled() {
        let error = Error::Cancelled;
        assert!(error.to_string().contains("Operation cancelled"));
    }

    #[test]
    fn test_error_other() {
        let error = Error::Other("Generic error".to_string());
        assert_eq!(error.to_string(), "Generic error");
    }

    #[test]
    fn test_error_multipart_upload() {
        let error = Error::MultipartUpload("Upload failed".to_string());
        assert!(error.to_string().contains("Multipart upload error"));
        assert!(error.to_string().contains("Upload failed"));
    }

    #[test]
    fn test_error_presigned_url_config() {
        let error = Error::PresignedUrlConfig("Invalid expiration".to_string());
        assert!(error
            .to_string()
            .contains("Invalid presigned URL configuration"));
        assert!(error.to_string().contains("Invalid expiration"));
    }

    #[test]
    fn test_error_cors_config() {
        let error = Error::CorsConfig("Invalid CORS rule".to_string());
        assert!(error.to_string().contains("CORS configuration error"));
        assert!(error.to_string().contains("Invalid CORS rule"));
    }

    #[test]
    fn test_error_lifecycle_rule() {
        let error = Error::LifecycleRule("Invalid rule".to_string());
        assert!(error.to_string().contains("Lifecycle rule error"));
        assert!(error.to_string().contains("Invalid rule"));
    }

    #[test]
    fn test_error_bucket_settings() {
        let error = Error::BucketSettings("Invalid settings".to_string());
        assert!(error.to_string().contains("Bucket settings error"));
        assert!(error.to_string().contains("Invalid settings"));
    }
}
