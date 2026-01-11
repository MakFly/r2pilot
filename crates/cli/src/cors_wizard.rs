//! Interactive wizard for CORS configuration

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect};
use r2pilot_core::{BucketCorsConfig, CorsRule};

/// Run the interactive CORS configuration wizard
pub async fn run_cors_wizard() -> Result<BucketCorsConfig> {
    let theme = ColorfulTheme::default();

    println!("🔧 CORS Configuration Wizard");
    println!();

    // Ask if they want to allow all origins
    let allow_all = Confirm::with_theme(&theme)
        .with_prompt("Allow requests from all origins ( wildcard)?")
        .default(true)
        .interact()?;

    let allowed_origins = if allow_all {
        vec!["*".to_string()]
    } else {
        let input: String = Input::with_theme(&theme)
            .with_prompt("Enter allowed origins (comma-separated, e.g., https://example.com,https://app.example.com)")
            .allow_empty(false)
            .interact()?;
        input.split(',').map(|s| s.trim().to_string()).collect()
    };

    // Select allowed methods
    let methods_selection = MultiSelect::with_theme(&theme)
        .with_prompt("Select allowed HTTP methods")
        .items(&["GET", "PUT", "POST", "DELETE", "HEAD", "OPTIONS"])
        .defaults(&[true, false, false, false, false, true])
        .interact()?;

    let allowed_methods: Vec<String> = methods_selection.iter().map(|&s| s.to_string()).collect();

    // Allowed headers
    let allow_all_headers = Confirm::with_theme(&theme)
        .with_prompt("Allow all headers ( wildcard)?")
        .default(true)
        .interact()?;

    let allowed_headers = if allow_all_headers {
        Some(vec!["*".to_string()])
    } else {
        let input: String = Input::with_theme(&theme)
            .with_prompt(
                "Enter allowed headers (comma-separated, e.g., Content-Type,Authorization)",
            )
            .allow_empty(true)
            .interact()?;
        if input.is_empty() {
            None
        } else {
            Some(
                input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>(),
            )
        }
    };

    // Max age
    let max_age_input: String = Input::with_theme(&theme)
        .with_prompt("Cache preflight response for how many seconds? (0 = no caching)")
        .default("86400".to_string())
        .interact()?;

    let max_age_seconds = max_age_input.parse::<u64>().ok();

    // Build rule
    let rule = CorsRule {
        allowed_origins,
        allowed_methods,
        allowed_headers,
        max_age_seconds,
    };

    let config = BucketCorsConfig { rules: vec![rule] };

    println!();
    println!("✅ CORS configuration created!");
    println!();
    println!("Allowed origins: {:?}", config.rules[0].allowed_origins);
    println!("Allowed methods: {:?}", config.rules[0].allowed_methods);
    println!("Allowed headers: {:?}", config.rules[0].allowed_headers);
    println!("Max age: {:?}", config.rules[0].max_age_seconds);

    Ok(config)
}

/// Create a CORS config from JSON file
pub async fn load_cors_from_file(file_path: &str) -> Result<BucketCorsConfig> {
    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;

    let config: BucketCorsConfig = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse CORS config: {}", e))?;

    Ok(config)
}

/// Save CORS config to JSON file
pub async fn save_cors_to_file(file_path: &str, config: &BucketCorsConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

    tokio::fs::write(file_path, json)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write file {}: {}", file_path, e))?;

    Ok(())
}
