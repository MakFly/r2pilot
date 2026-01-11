//! Interactive setup wizard for r2pilot configuration

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use indicatif::{ProgressBar, ProgressStyle};
use r2pilot_core::{save_config, CloudflareConfig, ConfigFile, R2Config};

/// Run the interactive setup wizard
pub async fn run_init_wizard() -> Result<()> {
    println!("🚀 Welcome to r2pilot setup!\n");

    println!("This wizard will guide you through the configuration process.");
    println!("You will need:");
    println!("  1. Your Cloudflare Account ID");
    println!("  2. An API Token OR Access Key ID + Secret Access Key");
    println!("  3. Your R2 bucket name\n");

    // Step 1: Account ID
    let account_id = prompt_account_id()?;

    // Step 2: Choose auth method
    let (api_token, access_key_id, secret_access_key) = prompt_auth_method()?;

    // Step 3: Build endpoint from account_id
    let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);

    // Step 4: Default bucket
    let default_bucket = prompt_bucket_name()?;

    // Summary
    println!("\n📋 Configuration summary:");
    println!("  Account ID: {}", account_id);
    println!("  Endpoint: {}", endpoint);
    println!("  Bucket: {}", default_bucket);
    println!(
        "  Auth: {}",
        if api_token.is_some() {
            "API Token"
        } else {
            "Access Keys"
        }
    );

    // Confirmation
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Save this configuration?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("❌ Configuration cancelled");
        return Ok(());
    }

    // Create config
    let config = ConfigFile {
        cloudflare: CloudflareConfig {
            account_id: account_id.clone(),
            endpoint: endpoint.clone(),
            api_token: api_token.clone(),
            access_key_id: access_key_id.clone(),
            secret_access_key: secret_access_key.clone(),
        },
        r2: R2Config {
            default_bucket: default_bucket.clone(),
            region: "auto".to_string(),
            default_expiration: 7200,
        },
        advanced: None,
        logging: None,
        output: None,
    };

    // Save config
    let pb = ProgressBar::new(2);
    pb.set_style(
        ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] {msg}")?,
    );
    pb.set_message("Saving configuration...");

    save_config(&config)?;

    pb.inc(1);
    pb.finish_with_message("✅ Configuration saved!");

    println!("\n🎉 Setup complete!");
    println!("\nConfiguration saved to: ~/.config/r2pilot/config.toml");
    println!("\nYou can now use r2pilot:");
    println!("  $ r2pilot buckets list");
    println!("  $ r2pilot files upload file.txt path/to/file.txt");
    println!("  $ r2pilot config show");

    Ok(())
}

/// Prompt for Cloudflare Account ID
fn prompt_account_id() -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Cloudflare Account ID")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.is_empty() {
                Err("Account ID cannot be empty")
            } else if !input.chars().all(|c| c.is_ascii_alphanumeric()) {
                Err("Account ID must be alphanumeric (32 characters)")
            } else if input.len() != 32 {
                Err("Account ID must be exactly 32 characters")
            } else {
                Ok(())
            }
        })
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get Account ID: {}", e))
}

/// Prompt for authentication method
fn prompt_auth_method() -> Result<(Option<String>, Option<String>, Option<String>)> {
    let auth_methods = vec![
        "API Token (recommended)",
        "Access Key ID + Secret Access Key",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Authentication method")
        .items(&auth_methods)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to select auth method: {}", e))?;

    match selection {
        0 => {
            // API Token
            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("API Token")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.is_empty() {
                        Err("API Token cannot be empty")
                    } else if !input.starts_with("eyJ") {
                        Err("Invalid token format (should start with 'eyJ')")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .map_err(|e| anyhow::anyhow!("Failed to get API Token: {}", e))?;

            Ok((Some(token), None, None))
        }
        1 => {
            // Access Keys
            let access_key = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Access Key ID")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.is_empty() {
                        Err("Access Key ID cannot be empty")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .map_err(|e| anyhow::anyhow!("Failed to get Access Key ID: {}", e))?;

            let secret_key = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Secret Access Key")
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.is_empty() {
                        Err("Secret Access Key cannot be empty")
                    } else if input.len() < 20 {
                        Err("Secret Access Key seems too short")
                    } else {
                        Ok(())
                    }
                })
                .interact()
                .map_err(|e| anyhow::anyhow!("Failed to get Secret Access Key: {}", e))?;

            Ok((None, Some(access_key), Some(secret_key)))
        }
        _ => unreachable!(),
    }
}

/// Prompt for default bucket name
fn prompt_bucket_name() -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Default bucket name")
        .default("my-bucket".to_string())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.is_empty() {
                Err("Bucket name cannot be empty")
            } else if input.len() < 3 {
                Err("Bucket name must be at least 3 characters")
            } else if input.len() > 63 {
                Err("Bucket name must be less than 64 characters")
            } else if !input
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '.')
            {
                Err("Bucket name can only contain alphanumeric characters, hyphens, and dots")
            } else {
                Ok(())
            }
        })
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to get bucket name: {}", e))
}
