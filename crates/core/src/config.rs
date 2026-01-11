//! Configuration management for r2pilot

use crate::error::{Error, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration directory name
const CONFIG_DIR: &str = "r2pilot";

/// Configuration file name
const CONFIG_FILE: &str = "config.toml";

/// Credentials file name (separate from config for security)
const CREDENTIALS_FILE: &str = "credentials.toml";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub cloudflare: CloudflareConfig,
    pub r2: R2Config,
    pub advanced: Option<AdvancedConfig>,
    pub logging: Option<LoggingConfig>,
    pub output: Option<OutputConfig>,
}

/// Cloudflare configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub account_id: String,
    pub endpoint: String,

    // API Token (preferred method)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,

    // OR Access Keys (alternative method)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
}

/// R2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Config {
    pub default_bucket: String,
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_expiration")]
    pub default_expiration: u64,
}

/// Advanced configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_uploads: usize,
    /// Multipart upload chunk size in MB (default: 100)
    #[serde(default = "default_multipart_chunk_size")]
    pub multipart_chunk_size_mb: usize,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            retry_delay: default_retry_delay(),
            max_concurrent_uploads: default_max_concurrent(),
            multipart_chunk_size_mb: default_multipart_chunk_size(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_format")]
    pub default_format: String,
    #[serde(default = "default_color")]
    pub color: String,
}

// Default values
fn default_region() -> String {
    "auto".to_string()
}

fn default_expiration() -> u64 {
    7200 // 2 hours
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_delay() -> u64 {
    1000 // 1 second
}

fn default_max_concurrent() -> usize {
    5
}

fn default_multipart_chunk_size() -> usize {
    100 // 100MB
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_output_format() -> String {
    "table".to_string()
}

fn default_color() -> String {
    "auto".to_string()
}

/// Get the configuration directory
pub fn get_config_dir() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| Error::Config("Cannot determine home directory".to_string()))?;
    let config_dir = home.join(".config").join(CONFIG_DIR);

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| Error::Config(format!("Failed to create config directory: {}", e)))?;
    }

    Ok(config_dir)
}

/// Get the configuration file path
pub fn get_config_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(CONFIG_FILE))
}

/// Get the credentials file path
pub fn get_credentials_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(CREDENTIALS_FILE))
}

/// Load configuration from file
pub fn load_config() -> Result<ConfigFile> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(Error::ConfigNotFound(config_path));
    }

    let content = fs::read_to_string(&config_path).map_err(|e| {
        Error::InvalidConfig(format!("Failed to read config file: {}", e))
    })?;

    let config: ConfigFile = toml::from_str(&content).map_err(|e| {
        Error::InvalidConfig(format!("Failed to parse config file: {}", e))
    })?;

    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &ConfigFile) -> Result<()> {
    let config_path = get_config_path()?;

    let content = toml::to_string_pretty(config).map_err(|e| {
        Error::InvalidConfig(format!("Failed to serialize config: {}", e))
    })?;

    fs::write(&config_path, content).map_err(|e| {
        Error::Config(format!("Failed to write config file: {}", e))
    })?;

    // Set secure permissions on config file (read/write for owner only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&config_path, perms)?;
    }

    Ok(())
}

/// Validate configuration
pub fn validate_config(config: &ConfigFile) -> Result<()> {
    // Validate account_id (should be 32 characters)
    if config.cloudflare.account_id.len() != 32 {
        return Err(Error::InvalidInput(format!(
            "Invalid Account ID format (expected 32 characters, got {})",
            config.cloudflare.account_id.len()
        )));
    }

    // Validate authentication method
    let has_api_token = config.cloudflare.api_token.is_some();
    let has_access_keys = config.cloudflare.access_key_id.is_some()
        && config.cloudflare.secret_access_key.is_some();

    if !has_api_token && !has_access_keys {
        return Err(Error::Config(
            "No authentication method configured. Either api_token or access_key_id + secret_access_key must be set".to_string()
        ));
    }

    // Validate bucket name
    if config.r2.default_bucket.is_empty() {
        return Err(Error::InvalidInput("Bucket name cannot be empty".to_string()));
    }

    // Validate expiration time
    if config.r2.default_expiration > 604800 {
        // 7 days in seconds
        return Err(Error::InvalidInput(
            "Default expiration cannot exceed 7 days (604800 seconds)".to_string()
        ));
    }

    Ok(())
}

/// Check if configuration exists
pub fn config_exists() -> bool {
    get_config_path().map(|p| p.exists()).unwrap_or(false)
}

/// Public alias for ConfigFile (used by lib.rs)
pub use ConfigFile as Config;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_valid_config() -> ConfigFile {
        ConfigFile {
            cloudflare: CloudflareConfig {
                account_id: "0123456789abcdef0123456789abcdef".to_string(),
                api_token: Some("test_token".to_string()),
                endpoint: "https://test.r2.cloudflarestorage.com".to_string(),
                access_key_id: None,
                secret_access_key: None,
            },
            r2: R2Config {
                default_bucket: "test-bucket".to_string(),
                region: "auto".to_string(),
                default_expiration: 3600,
            },
            advanced: None,
            logging: None,
            output: None,
        }
    }

    #[test]
    fn test_validate_config_valid() {
        let config = make_valid_config();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_invalid_account_id() {
        let mut config = make_valid_config();
        config.cloudflare.account_id = "too_short".to_string();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_no_auth() {
        let mut config = make_valid_config();
        config.cloudflare.api_token = None;
        config.cloudflare.access_key_id = None;
        config.cloudflare.secret_access_key = None;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_with_access_keys() {
        let mut config = make_valid_config();
        config.cloudflare.api_token = None;
        config.cloudflare.access_key_id = Some("test_key_id".to_string());
        config.cloudflare.secret_access_key = Some("test_secret".to_string());
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_empty_bucket() {
        let mut config = make_valid_config();
        config.r2.default_bucket = "".to_string();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_expiration_too_long() {
        let mut config = make_valid_config();
        config.r2.default_expiration = 604801; // More than 7 days
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_expiration_boundary() {
        let mut config = make_valid_config();
        config.r2.default_expiration = 604800; // Exactly 7 days
        assert!(validate_config(&config).is_ok());
    }
}
