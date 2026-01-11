//! r2pilot-core - Core library for R2 Pilot CLI
//!
//! This library provides the core functionality for managing Cloudflare R2 storage,
//! including configuration management, R2 client operations, and Cloudflare API integration.

pub mod client;
pub mod cloudflare;
pub mod config;
pub mod error;
pub mod presigned;

// Re-export commonly used types
pub use client::{
    requires_multipart_upload, CompletedPart, MultipartUploadConfig, MultipartUploadProgress,
    R2Client,
};
pub use cloudflare::{
    ApiToken, BucketCorsConfig, CloudflareClient, CorsRule, ErrorDocument, IndexDocument,
    LifecycleConfiguration, LifecycleExpiration, LifecycleFilter, LifecycleRule, R2Bucket,
    R2TokenBuilder, WebsiteConfiguration,
};
pub use config::{config_exists, get_config_path, load_config, save_config, validate_config};
pub use config::{CloudflareConfig, Config, ConfigFile, R2Config};
pub use error::{Error, Result};
pub use presigned::{generate_presigned_url, PresignedMethod, PresignedUrlConfig};
