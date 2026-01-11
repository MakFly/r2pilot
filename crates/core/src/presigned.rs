//! Presigned URL generation for R2

use crate::error::{Error, Result};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// HTTP methods for presigned URLs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresignedMethod {
    Get,
    Put,
    Delete,
}

impl PresignedMethod {
    pub fn as_str(&self) -> &str {
        match self {
            PresignedMethod::Get => "GET",
            PresignedMethod::Put => "PUT",
            PresignedMethod::Delete => "DELETE",
        }
    }
}

impl std::fmt::Display for PresignedMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Configuration for generating presigned URLs
#[derive(Debug, Clone)]
pub struct PresignedUrlConfig {
    /// HTTP method
    pub method: PresignedMethod,
    /// Object key
    pub key: String,
    /// Expiration time
    pub expires_in: Duration,
    /// Content type (for PUT requests)
    pub content_type: Option<String>,
}

impl PresignedUrlConfig {
    /// Create a new presigned URL configuration
    pub fn new(method: PresignedMethod, key: String, expires_in: Duration) -> Self {
        Self {
            method,
            key,
            expires_in,
            content_type: None,
        }
    }

    /// Set content type (useful for PUT requests)
    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }
}

/// Generate a presigned URL for R2
///
/// Note: This is a simplified implementation. For proper AWS SigV4 signing,
/// additional setup is required. This method returns a basic URL structure
/// that can be used with direct access credentials.
pub fn generate_presigned_url(
    endpoint: &str,
    bucket: &str,
    key: &str,
    config: PresignedUrlConfig,
) -> Result<String> {
    // Parse endpoint to get host
    let endpoint_uri: http::Uri = endpoint.parse()
        .map_err(|e| Error::PresignedUrlConfig(format!("Invalid endpoint URL: {}", e)))?;

    let host = endpoint_uri.host()
        .ok_or_else(|| Error::PresignedUrlConfig("Endpoint has no host".to_string()))?;

    // Build the object URL
    let url = format!("https://{}/{}/{}", host, bucket, key);

    // Calculate expiration timestamp
    let expires_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::PresignedUrlConfig(format!("Time error: {}", e)))?
        .as_secs()
        + config.expires_in.as_secs();

    // Add method parameter for PUT/DELETE
    let url = match config.method {
        PresignedMethod::Get => url,
        PresignedMethod::Put => format!("{}?method=PUT", url),
        PresignedMethod::Delete => format!("{}?method=DELETE", url),
    };

    // Build final URL with expiration
    Ok(format!("{}&expires={}", url, expires_timestamp))
}

/// Generate a presigned GET URL for downloading
pub fn generate_presigned_get_url(
    endpoint: &str,
    bucket: &str,
    key: &str,
    expires_in: Duration,
) -> Result<String> {
    let config = PresignedUrlConfig::new(PresignedMethod::Get, key.to_string(), expires_in);
    generate_presigned_url(endpoint, bucket, key, config)
}

/// Generate a presigned PUT URL for uploading
pub fn generate_presigned_put_url(
    endpoint: &str,
    bucket: &str,
    key: &str,
    expires_in: Duration,
    content_type: &str,
) -> Result<String> {
    let config = PresignedUrlConfig::new(PresignedMethod::Put, key.to_string(), expires_in)
        .with_content_type(content_type.to_string());
    generate_presigned_url(endpoint, bucket, key, config)
}

/// Generate a presigned DELETE URL
pub fn generate_presigned_delete_url(
    endpoint: &str,
    bucket: &str,
    key: &str,
    expires_in: Duration,
) -> Result<String> {
    let config = PresignedUrlConfig::new(PresignedMethod::Delete, key.to_string(), expires_in);
    generate_presigned_url(endpoint, bucket, key, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presigned_method_display() {
        assert_eq!(PresignedMethod::Get.as_str(), "GET");
        assert_eq!(PresignedMethod::Put.as_str(), "PUT");
        assert_eq!(PresignedMethod::Delete.as_str(), "DELETE");
    }

    #[test]
    fn test_presigned_url_config() {
        let config = PresignedUrlConfig::new(
            PresignedMethod::Put,
            "test-key.txt".to_string(),
            Duration::from_secs(3600),
        );

        assert_eq!(config.method, PresignedMethod::Put);
        assert_eq!(config.key, "test-key.txt");
        assert_eq!(config.expires_in.as_secs(), 3600);
        assert!(config.content_type.is_none());

        let config_with_ct = config.with_content_type("text/plain".to_string());
        assert_eq!(config_with_ct.content_type, Some("text/plain".to_string()));
    }
}
