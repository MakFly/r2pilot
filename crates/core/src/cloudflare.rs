//! Cloudflare API client for managing R2 and API tokens

use crate::error::{Error, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};

/// Cloudflare API client
pub struct CloudflareClient {
    api_token: String,
    account_id: String,
    http_client: Client,
    base_url: String,
}

impl CloudflareClient {
    /// Create a new Cloudflare client
    pub fn new(api_token: String, account_id: String) -> Self {
        Self {
            api_token,
            account_id,
            http_client: Client::new(),
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
        }
    }

    /// List all API tokens
    pub async fn list_tokens(&self) -> Result<Vec<ApiToken>> {
        let response = self
            .http_client
            .get(&format!("{}/user/tokens", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a new API token
    pub async fn create_token(&self, params: CreateTokenParams) -> Result<ApiToken> {
        let response = self
            .http_client
            .post(&format!("{}/user/tokens", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&params)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Revoke an API token
    pub async fn revoke_token(&self, token_id: &str) -> Result<()> {
        let response = self
            .http_client
            .delete(&format!("{}/user/tokens/{}", self.base_url, token_id))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    /// List all R2 buckets
    pub async fn list_buckets(&self) -> Result<Vec<R2Bucket>> {
        let response = self
            .http_client
            .get(&format!(
                "{}/accounts/{}/r2/buckets",
                self.base_url, self.account_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get details of a specific bucket
    pub async fn get_bucket(&self, name: &str) -> Result<R2Bucket> {
        let response = self
            .http_client
            .get(&format!(
                "{}/accounts/{}/r2/buckets/{}",
                self.base_url, self.account_id, name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a new R2 bucket
    pub async fn create_bucket(&self, name: &str, location: &str) -> Result<R2Bucket> {
        let body = serde_json::json!({
            "name": name,
            "location": {
                "location": location
            }
        });

        let response = self
            .http_client
            .post(&format!(
                "{}/accounts/{}/r2/buckets",
                self.base_url, self.account_id
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Delete an R2 bucket
    pub async fn delete_bucket(&self, name: &str) -> Result<()> {
        let response = self
            .http_client
            .delete(&format!(
                "{}/accounts/{}/r2/buckets/{}",
                self.base_url, self.account_id, name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    // === CORS Configuration ===

    /// Get CORS configuration for a bucket
    pub async fn get_bucket_cors(&self, bucket_name: &str) -> Result<BucketCorsConfig> {
        let response = self
            .http_client
            .get(&format!(
                "{}/accounts/{}/r2/buckets/{}/cors",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Set CORS configuration for a bucket
    pub async fn put_bucket_cors(
        &self,
        bucket_name: &str,
        config: &BucketCorsConfig,
    ) -> Result<()> {
        let response = self
            .http_client
            .put(&format!(
                "{}/accounts/{}/r2/buckets/{}/cors",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(config)
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    /// Delete CORS configuration for a bucket
    pub async fn delete_bucket_cors(&self, bucket_name: &str) -> Result<()> {
        let response = self
            .http_client
            .delete(&format!(
                "{}/accounts/{}/r2/buckets/{}/cors",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    // === Lifecycle Rules ===

    /// Get lifecycle rules for a bucket
    pub async fn get_bucket_lifecycle(&self, bucket_name: &str) -> Result<LifecycleConfiguration> {
        let response = self
            .http_client
            .get(&format!(
                "{}/accounts/{}/r2/buckets/{}/lifecycle",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Set lifecycle rules for a bucket
    pub async fn put_bucket_lifecycle(
        &self,
        bucket_name: &str,
        config: &LifecycleConfiguration,
    ) -> Result<()> {
        let response = self
            .http_client
            .put(&format!(
                "{}/accounts/{}/r2/buckets/{}/lifecycle",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(config)
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    /// Delete lifecycle rules for a bucket
    pub async fn delete_bucket_lifecycle(&self, bucket_name: &str) -> Result<()> {
        let response = self
            .http_client
            .delete(&format!(
                "{}/accounts/{}/r2/buckets/{}/lifecycle",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    // === Public Bucket / Website Configuration ===

    /// Enable static hosting for a bucket
    pub async fn put_bucket_website(
        &self,
        bucket_name: &str,
        config: &WebsiteConfiguration,
    ) -> Result<()> {
        let response = self
            .http_client
            .put(&format!(
                "{}/accounts/{}/r2/buckets/{}/website",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(config)
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    /// Get website configuration for a bucket
    pub async fn get_bucket_website(&self, bucket_name: &str) -> Result<WebsiteConfiguration> {
        let response = self
            .http_client
            .get(&format!(
                "{}/accounts/{}/r2/buckets/{}/website",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Disable static hosting for a bucket
    pub async fn delete_bucket_website(&self, bucket_name: &str) -> Result<()> {
        let response = self
            .http_client
            .delete(&format!(
                "{}/accounts/{}/r2/buckets/{}/website",
                self.base_url, self.account_id, bucket_name
            ))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        self.handle_response::<()>(response).await?;
        Ok(())
    }

    /// Handle API response
    async fn handle_response<T: for<'de> Deserialize<'de>>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let cloudflare_response: CloudflareResponse<T> = response.json().await?;
            if cloudflare_response.success {
                Ok(cloudflare_response.result)
            } else {
                let errors = cloudflare_response
                    .errors
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<_>>()
                    .join("; ");
                Err(Error::CloudflareApi(errors))
            }
        } else if status.as_u16() == 401 {
            Err(Error::Authentication("Invalid API token".to_string()))
        } else if status.as_u16() == 403 {
            Err(Error::PermissionDenied(
                "Insufficient permissions".to_string(),
            ))
        } else if status.as_u16() == 404 {
            Err(Error::NotFound("Resource not found".to_string()))
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(Error::CloudflareApi(format!(
                "HTTP {}: {}",
                status.as_u16(),
                error_text
            )))
        }
    }
}

/// Cloudflare API response wrapper
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    #[allow(dead_code)]
    messages: Vec<CloudflareMessage>,
    result: T,
}

/// Cloudflare error
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CloudflareError {
    code: i32,
    message: String,
}

/// Cloudflare message
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CloudflareMessage {
    code: i32,
    message: String,
}

/// API Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub id: String,
    pub name: String,
    pub status: String,
    pub issued_on: String,
    pub modified_on: String,
    pub expires_on: Option<String>,
    pub permissions: Vec<Permission>,
}

/// Permission structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub policy: PermissionPolicy,
}

/// Permission policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    pub permission_groups: Vec<PermissionGroup>,
    pub resources: Resources,
}

/// Permission group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub id: String,
    pub name: String,
}

/// Resources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountResources>,
}

/// Account resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
}

/// Parameters for creating a token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenParams {
    pub name: String,
    pub policy: TokenPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<TokenCondition>,
}

/// Token policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPolicy {
    pub permission_groups: Vec<PermissionGroup>,
    pub resources: Resources,
}

/// Token condition (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenCondition {
    pub request: Option<RequestCondition>,
}

/// Request condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCondition {
    pub ip: Option<IpCondition>,
}

/// IP condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpCondition {
    pub in_list: Option<Vec<String>>,
}

/// R2 Bucket information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Bucket {
    pub name: String,
    pub location: String,
    pub creation_date: String,
}

// === CORS Configuration Types ===

/// CORS configuration for a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketCorsConfig {
    pub rules: Vec<CorsRule>,
}

/// CORS rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorsRule {
    #[serde(rename = "allowedOrigins")]
    pub allowed_origins: Vec<String>,
    #[serde(rename = "allowedMethods")]
    pub allowed_methods: Vec<String>,
    #[serde(rename = "allowedHeaders")]
    pub allowed_headers: Option<Vec<String>>,
    #[serde(rename = "maxAgeSeconds")]
    pub max_age_seconds: Option<u64>,
}

// === Lifecycle Configuration Types ===

/// Lifecycle configuration for a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleConfiguration {
    pub rules: Vec<LifecycleRule>,
}

/// Lifecycle rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRule {
    pub id: String,
    pub filter: LifecycleFilter,
    pub status: String, // "Enabled" or "Disabled"
    pub expiration: Option<LifecycleExpiration>,
}

/// Lifecycle filter
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LifecycleFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

/// Lifecycle expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleExpiration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days: Option<u32>,
}

// === Website Configuration Types ===

/// Website configuration for static hosting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebsiteConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_document: Option<IndexDocument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_document: Option<ErrorDocument>,
}

/// Index document configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    pub suffix: String,
}

/// Error document configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDocument {
    pub key: String,
}

/// Builder for creating R2 tokens with edit permissions
pub struct R2TokenBuilder {
    name: String,
    account_id: String,
    ip_whitelist: Option<Vec<String>>,
}

impl R2TokenBuilder {
    /// Create a new token builder
    pub fn new(name: String, account_id: String) -> Self {
        Self {
            name,
            account_id,
            ip_whitelist: None,
        }
    }

    /// Set IP whitelist
    pub fn ip_whitelist(mut self, ips: Vec<String>) -> Self {
        self.ip_whitelist = Some(ips);
        self
    }

    /// Build the token creation parameters
    pub fn build(self) -> CreateTokenParams {
        CreateTokenParams {
            name: self.name,
            policy: TokenPolicy {
                permission_groups: vec![
                    // R2 Edit permission group
                    PermissionGroup {
                        id: "c4259685b71d4e928c3201fc048494ab".to_string(), // R2 Edit template ID
                        name: "Cloudflare R2 Edit".to_string(),
                    },
                ],
                resources: Resources {
                    account: Some(AccountResources {
                        include: Some(vec![self.account_id]),
                    }),
                },
            },
            condition: self.ip_whitelist.map(|ips| TokenCondition {
                request: Some(RequestCondition {
                    ip: Some(IpCondition { in_list: Some(ips) }),
                }),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r2_token_builder() {
        let builder = R2TokenBuilder::new("Test Token".to_string(), "abc123def456".to_string());

        let params = builder
            .ip_whitelist(vec!["192.168.1.1".to_string()])
            .build();

        assert_eq!(params.name, "Test Token");
        assert_eq!(params.policy.permission_groups.len(), 1);
        assert!(params.condition.is_some());
    }

    #[test]
    fn test_cloudflare_client_creation() {
        let api_token = "test-api-token".to_string();
        let account_id = "test-account-id".to_string();

        let client = CloudflareClient::new(api_token.clone(), account_id.to_string());

        assert_eq!(client.account_id, "test-account-id");
        assert_eq!(client.api_token, "test-api-token");
    }

    #[test]
    fn test_r2_bucket_creation() {
        let bucket = R2Bucket {
            name: "test-bucket".to_string(),
            location: "wnam".to_string(),
            creation_date: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(bucket.name, "test-bucket");
        assert_eq!(bucket.location, "wnam");
        assert_eq!(bucket.creation_date, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_cors_rule() {
        let rule = CorsRule {
            allowed_origins: vec!["https://example.com".to_string()],
            allowed_methods: vec!["GET".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allowed_headers: Some(vec!["*".to_string()]),
            max_age_seconds: Some(3600),
        };

        assert_eq!(rule.allowed_origins, vec!["https://example.com"]);
        assert_eq!(rule.allowed_methods, vec!["GET", "PUT", "DELETE"]);
        assert_eq!(rule.allowed_headers, Some(vec!["*".to_string()]));
        assert_eq!(rule.max_age_seconds, Some(3600));
    }

    #[test]
    fn test_bucket_cors_config() {
        let config = BucketCorsConfig {
            rules: vec![CorsRule {
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec!["GET".to_string(), "HEAD".to_string()],
                allowed_headers: None,
                max_age_seconds: None,
            }],
        };

        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].allowed_origins, vec!["*"]);
        assert_eq!(config.rules[0].allowed_methods, vec!["GET", "HEAD"]);
    }

    #[test]
    fn test_lifecycle_filter() {
        let filter = LifecycleFilter {
            prefix: Some("logs/".to_string()),
        };

        assert_eq!(filter.prefix, Some("logs/".to_string()));
    }

    #[test]
    fn test_lifecycle_expiration() {
        let expiration = LifecycleExpiration { days: Some(30) };

        assert_eq!(expiration.days, Some(30));
    }

    #[test]
    fn test_lifecycle_rule() {
        let rule = LifecycleRule {
            id: "log-rotation".to_string(),
            filter: LifecycleFilter {
                prefix: Some("logs/".to_string()),
            },
            status: "Enabled".to_string(),
            expiration: Some(LifecycleExpiration { days: Some(30) }),
        };

        assert_eq!(rule.id, "log-rotation");
        assert_eq!(rule.status, "Enabled");
        assert_eq!(rule.filter.prefix, Some("logs/".to_string()));
        assert_eq!(rule.expiration.unwrap().days, Some(30));
    }

    #[test]
    fn test_lifecycle_configuration() {
        let config = LifecycleConfiguration {
            rules: vec![LifecycleRule {
                id: "delete-old-videos".to_string(),
                filter: LifecycleFilter {
                    prefix: Some("videos/".to_string()),
                },
                status: "Enabled".to_string(),
                expiration: Some(LifecycleExpiration { days: Some(90) }),
            }],
        };

        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].id, "delete-old-videos");
    }

    #[test]
    fn test_index_document() {
        let index = IndexDocument {
            suffix: "index.html".to_string(),
        };

        assert_eq!(index.suffix, "index.html");
    }

    #[test]
    fn test_error_document() {
        let error = ErrorDocument {
            key: "404.html".to_string(),
        };

        assert_eq!(error.key, "404.html");
    }

    #[test]
    fn test_website_configuration() {
        let config = WebsiteConfiguration {
            index_document: Some(IndexDocument {
                suffix: "index.html".to_string(),
            }),
            error_document: Some(ErrorDocument {
                key: "error.html".to_string(),
            }),
        };

        assert_eq!(config.index_document.unwrap().suffix, "index.html");
        assert_eq!(config.error_document.unwrap().key, "error.html");
    }
}
