//! r2pilot-core - Core library for R2 Pilot CLI
//!
//! This library provides the core functionality for managing Cloudflare R2 storage,
//! including configuration management, R2 client operations, and Cloudflare API integration.

pub mod client;
pub mod cloudflare;
pub mod config;
pub mod error;

// Re-export commonly used types
pub use client::R2Client;
pub use cloudflare::{CloudflareClient, R2TokenBuilder, R2Bucket, ApiToken};
pub use config::{Config, ConfigFile, R2Config, CloudflareConfig};
pub use config::{load_config, save_config, validate_config, config_exists, get_config_path};
pub use error::{Error, Result};
