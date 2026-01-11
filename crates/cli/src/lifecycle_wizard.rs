//! Interactive wizard for Lifecycle rules configuration

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use r2pilot_core::{LifecycleConfiguration, LifecycleExpiration, LifecycleFilter, LifecycleRule};

/// Run the interactive Lifecycle configuration wizard
pub async fn run_lifecycle_wizard() -> Result<LifecycleConfiguration> {
    let theme = ColorfulTheme::default();

    println!("🗓️  Lifecycle Rules Configuration Wizard");
    println!();

    let mut rules = Vec::new();

    loop {
        println!("Add a new lifecycle rule...");

        // Rule ID
        let id = Input::with_theme(&theme)
            .with_prompt("Rule ID (unique identifier)")
            .default(format!("rule-{}", rules.len() + 1))
            .interact()?;

        // Prefix filter
        let prefix: String = Input::with_theme(&theme)
            .with_prompt("Object prefix (leave empty for all objects)")
            .allow_empty(true)
            .default(String::new())
            .interact()?;

        let filter = LifecycleFilter {
            prefix: if prefix.is_empty() {
                None
            } else {
                Some(prefix)
            },
        };

        // Expiration by days
        let use_expiration = Confirm::with_theme(&theme)
            .with_prompt("Add expiration to this rule?")
            .default(true)
            .interact()?;

        let expiration = if use_expiration {
            let days_input: String = Input::with_theme(&theme)
                .with_prompt("Delete objects after how many days?")
                .default("30".to_string())
                .interact()?;

            Some(LifecycleExpiration {
                days: Some(days_input.parse().unwrap_or(30)),
            })
        } else {
            None
        };

        // Status
        let enabled = Confirm::with_theme(&theme)
            .with_prompt("Enable this rule?")
            .default(true)
            .interact()?;

        let rule = LifecycleRule {
            id: id.clone(),
            filter,
            status: if enabled {
                "Enabled".to_string()
            } else {
                "Disabled".to_string()
            },
            expiration,
        };

        rules.push(rule);

        println!();
        println!("✅ Rule '{}' added!", id);

        // Add another rule?
        let add_more = Confirm::with_theme(&theme)
            .with_prompt("Add another rule?")
            .default(false)
            .interact()?;

        if !add_more {
            break;
        }
        println!();
    }

    let config = LifecycleConfiguration { rules };

    println!();
    println!("✅ Lifecycle configuration created!");
    println!();
    println!("Total rules: {}", config.rules.len());

    Ok(config)
}

/// Create a Lifecycle config from JSON file
pub async fn load_lifecycle_from_file(file_path: &str) -> Result<LifecycleConfiguration> {
    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;

    let config: LifecycleConfiguration = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse Lifecycle config: {}", e))?;

    Ok(config)
}

/// Save Lifecycle config to JSON file
#[allow(dead_code)]
pub async fn save_lifecycle_to_file(
    file_path: &str,
    config: &LifecycleConfiguration,
) -> Result<()> {
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;

    tokio::fs::write(file_path, json)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write file {}: {}", file_path, e))?;

    Ok(())
}
