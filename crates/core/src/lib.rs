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
pub use cloudflare::CloudflareClient;
pub use config::{Config, ConfigFile, R2Config};
pub use config::CloudflareConfig;
pub use error::{Error, Result};
