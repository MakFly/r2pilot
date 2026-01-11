//! Command handlers for r2pilot CLI

use clap::Command;
use clap_complete::{generate, Shell as ClapShell};
use crate::wizard::run_init_wizard;
use anyhow::Result;
use r2pilot_core::{
    R2Client,
    load_config, validate_config, get_config_path,
};
use tabled::{Table, Tabled};
use std::path::Path;

/// Handle init command
pub async fn handle_init() -> Result<()> {
    run_init_wizard().await
}

/// Handle config commands
pub async fn handle_config(action: &str) -> Result<()> {
    match action {
        "show" => {
            println!("Current configuration:");
            println!();

            let config = load_config()?;

            println!("Cloudflare:");
            println!("  Account ID: {}", &config.cloudflare.account_id[..8]);
            println!("  Endpoint: {}", config.cloudflare.endpoint);
            println!("  Auth: {}",
                if config.cloudflare.api_token.is_some() {
                    "API Token"
                } else {
                    "Access Keys"
                }
            );
            println!();
            println!("R2:");
            println!("  Default bucket: {}", config.r2.default_bucket);
            println!("  Region: {}", config.r2.region);
            println!("  Default expiration: {}s", config.r2.default_expiration);

            Ok(())
        }
        "validate" => {
            println!("Validating credentials...");

            let config = load_config()?;

            // Validate config format
            validate_config(&config)?;
            println!("  ✅ Valid configuration format");

            // Get R2 credentials
            let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
                return Err(anyhow::anyhow!(
                    "Validation via API Token not yet implemented.\n\
                     Use Access Keys in your configuration.\n\
                     Get your Access Keys from: https://dash.cloudflare.com/{}/r2/api-tokens",
                    config.cloudflare.account_id
                ));
            } else {
                (
                    config.cloudflare.access_key_id.clone()
                        .ok_or_else(|| anyhow::anyhow!("Access Key ID not configured (run 'r2pilot init')"))?,
                    config.cloudflare.secret_access_key.clone()
                        .ok_or_else(|| anyhow::anyhow!("Secret Access Key not configured (run 'r2pilot init')"))?,
                )
            };

            println!("  Testing R2 connection...");
            let r2_client = R2Client::new(
                config.cloudflare.endpoint.clone(),
                access_key_id,
                secret_access_key,
                config.r2.default_bucket.clone(),
            ).await?;

            // Try to list objects as a connection test
            let _objects = r2_client.list_objects(None).await?;

            println!("  ✅ Valid configuration!");
            println!("  ✅ R2 connection successful!");

            Ok(())
        }
        "edit" => {
            println!("Opening editor...");
            println!("  File: ~/.config/r2pilot/config.toml");
            println!();

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let config_path = get_config_path()?;

            let status = std::process::Command::new(editor)
                .arg(&config_path)
                .status()?;

            if status.success() {
                println!("  ✅ Configuration edited");

                // Validate after edit
                let config = load_config()?;
                validate_config(&config)?;
                println!("  ✅ Configuration valid");
            } else {
                println!("  ⚠️  Editor exited with error");
            }

            Ok(())
        }
        _ => {
            println!("Unknown action: {}", action);
            println!("Available actions: show, edit, validate");
            Ok(())
        }
    }
}

/// Handle tokens commands
pub async fn handle_tokens(action: &str, token_id: Option<&str>) -> Result<()> {
    use r2pilot_core::{CloudflareClient, R2TokenBuilder};

    let config = load_config()?;

    // Get API token for Cloudflare API access
    let api_token = config.cloudflare.api_token.clone().ok_or_else(|| anyhow::anyhow!(
        "API Token required to manage tokens.\n\
         Add 'api_token' to your configuration.\n\
         Get an API Token from: https://dash.cloudflare.com/profile/api-tokens"
    ))?;

    let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());

    match action {
        "list" => {
            println!("Listing Cloudflare API Tokens...");
            println!();

            let tokens = cf_client.list_tokens().await?;

            if tokens.is_empty() {
                println!("  No tokens found");
            } else {
                #[derive(Tabled)]
                struct TokenRow {
                    name: String,
                    status: String,
                    id: String,
                    issued: String,
                    expires: String,
                }

                let rows: Vec<TokenRow> = tokens.iter().map(|t| TokenRow {
                    name: t.name.clone(),
                    status: format_status(&t.status),
                    id: format_id(&t.id),
                    issued: format_date(&t.issued_on),
                    expires: t.expires_on.as_ref().map(|d| format_date(d)).unwrap_or_else(|| "Never".to_string()),
                }).collect();

                println!("{}", Table::new(rows));
            }

            Ok(())
        }
        "create" => {
            // Interactive prompt for token creation
            use dialoguer::{Input, theme::ColorfulTheme};

            let theme = ColorfulTheme::default();

            let name = Input::with_theme(&theme)
                .with_prompt("Token name")
                .default(format!("r2pilot-{}", chrono::Utc::now().format("%Y%m%d")))
                .interact()?;

            println!();
            println!("Creating token '{}'...", name);

            let builder = R2TokenBuilder::new(name.clone(), config.cloudflare.account_id.clone());
            let params = builder.build();

            let token = cf_client.create_token(params).await?;

            println!("  ✅ Token created: {}", token.name);
            println!();
            println!("  IMPORTANT: Copy this token now, it won't be shown again!");
            println!("  Status: {}", format_status(&token.status));
            println!();
            println!("  ⚠️  Save this token in your configuration:");
            println!("     api_token = \"<your_token>\"");

            Ok(())
        }
        "revoke" => {
            let id = token_id.ok_or_else(|| anyhow::anyhow!("Token ID required (use 'tokens list' to see IDs)"))?;

            println!("⚠️  Warning: you are about to revoke token '{}'", id);
            println!("  This action is IRREVERSIBLE!");

            cf_client.revoke_token(id).await?;

            println!("  ✅ Token revoked: {}", id);

            Ok(())
        }
        _ => {
            println!("Unknown action: {}", action);
            println!("Available actions: list, create, revoke");
            Ok(())
        }
    }
}

/// Format token status with emoji
fn format_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "active" => "✅ Active".to_string(),
        "revoked" => "❌ Revoked".to_string(),
        "expired" => "⏰ Expired".to_string(),
        _ => status.to_string(),
    }
}

/// Format ID to show only first 8 chars
fn format_id(id: &str) -> String {
    if id.len() > 8 {
        format!("{}...", &id[..8])
    } else {
        id.to_string()
    }
}

/// Handle buckets commands
pub async fn handle_buckets(action: &str, name: Option<&str>) -> Result<()> {
    use r2pilot_core::CloudflareClient;

    let config = load_config()?;

    match action {
        "list" => {
            // List buckets requires Cloudflare API token
            let api_token = config.cloudflare.api_token.clone().ok_or_else(|| anyhow::anyhow!(
                "API Token required to list buckets.\n\
                 Add 'api_token' to your configuration.\n\
                 Get an API Token from: https://dash.cloudflare.com/{}/r2/api-tokens",
                config.cloudflare.account_id
            ))?;

            println!("Listing R2 buckets...");
            println!();

            let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());
            let buckets = cf_client.list_buckets().await?;

            if buckets.is_empty() {
                println!("  No buckets found");
            } else {
                #[derive(Tabled)]
                struct BucketRow {
                    name: String,
                    location: String,
                    created: String,
                }

                let rows: Vec<BucketRow> = buckets.iter().map(|b| BucketRow {
                    name: b.name.clone(),
                    location: b.location.clone(),
                    created: format_date(&b.creation_date),
                }).collect();

                println!("{}", Table::new(rows));
            }
            println!();
            println!("Default bucket: {}", config.r2.default_bucket);

            Ok(())
        }
        "create" => {
            let bucket_name = name.ok_or_else(|| anyhow::anyhow!("Bucket name required"))?;

            let api_token = config.cloudflare.api_token.clone().ok_or_else(|| anyhow::anyhow!(
                "API Token required to create buckets.\n\
                 Add 'api_token' to your configuration.\n\
                 Get an API Token from: https://dash.cloudflare.com/{}/r2/api-tokens",
                config.cloudflare.account_id
            ))?;

            println!("Creating bucket '{}'...", bucket_name);

            let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());
            let bucket = cf_client.create_bucket(bucket_name, "eu").await?;

            println!("  ✅ Bucket created: {}", bucket.name);
            println!("  Location: {}", bucket.location);

            Ok(())
        }
        "delete" => {
            let bucket_name = name.ok_or_else(|| anyhow::anyhow!("Bucket name required"))?;

            // Prevent accidental deletion of default bucket
            if bucket_name == config.r2.default_bucket {
                return Err(anyhow::anyhow!(
                    "Cannot delete default bucket '{}'.\n\
                     Change the default bucket in your configuration first.",
                    bucket_name
                ));
            }

            let api_token = config.cloudflare.api_token.clone().ok_or_else(|| anyhow::anyhow!(
                "API Token required to delete buckets.\n\
                 Add 'api_token' to your configuration."
            ))?;

            println!("⚠️  Warning: you are about to delete bucket '{}'", bucket_name);
            println!("  This action is IRREVERSIBLE!");

            let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());
            cf_client.delete_bucket(bucket_name).await?;

            println!("  ✅ Bucket deleted: {}", bucket_name);

            Ok(())
        }
        "info" | "ls" => {
            // Get R2 credentials for S3 API access
            let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
                return Err(anyhow::anyhow!(
                    "The '{}' command requires R2 Access Keys.\n\
                     Configure access_key_id and secret_access_key to use S3 operations.",
                    action
                ));
            } else {
                (
                    config.cloudflare.access_key_id.clone()
                        .ok_or_else(|| anyhow::anyhow!("Access Key ID not configured (run 'r2pilot init')"))?,
                    config.cloudflare.secret_access_key.clone()
                        .ok_or_else(|| anyhow::anyhow!("Secret Access Key not configured (run 'r2pilot init')"))?,
                )
            };

            let bucket = name.unwrap_or(&config.r2.default_bucket);

            let r2_client = R2Client::new(
                config.cloudflare.endpoint.clone(),
                access_key_id,
                secret_access_key,
                bucket.to_string(),
            ).await?;

            if action == "info" {
                println!("Bucket '{}' information...", bucket);

                // Try to list objects as a connection test
                let objects = r2_client.list_objects(None).await?;

                println!("  Name: {}", bucket);
                println!("  Objects: {}", objects.len());
            } else {
                println!("Bucket '{}' contents...", bucket);

                let objects = r2_client.list_objects(None).await?;

                if objects.is_empty() {
                    println!("  Empty bucket");
                } else {
                    #[derive(Tabled)]
                    struct ObjectRow {
                        key: String,
                        size: String,
                    }

                    let rows: Vec<ObjectRow> = objects.iter().map(|o| ObjectRow {
                        key: o.key.clone(),
                        size: format_bytes(o.size),
                    }).collect();

                    println!();
                    println!("{}", Table::new(rows));
                }
            }

            Ok(())
        }
        _ => {
            println!("Unknown action: {}", action);
            println!("Available actions: list, create, delete, info, ls");
            Ok(())
        }
    }
}

/// Format ISO date string to readable format
fn format_date(iso_date: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(iso_date) {
        Ok(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        Err(_) => iso_date.to_string(),
    }
}

/// Handle files commands
pub async fn handle_files(
    action: &str,
    file: Option<&str>,
    key: Option<&str>,
    bucket: Option<&str>,
    prefix: Option<&str>,
    _progress: bool,
) -> Result<()> {
    let config = load_config()?;

    // Get R2 credentials
    let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
        return Err(anyhow::anyhow!(
            "API Token not supported yet.\n\
             Use Access Keys in your configuration.\n\
             Get your Access Keys from: https://dash.cloudflare.com/{}/r2/api-tokens",
            config.cloudflare.account_id
        ));
    } else {
        (
            config.cloudflare.access_key_id.clone().unwrap(),
            config.cloudflare.secret_access_key.clone().unwrap(),
        )
    };

    let bucket = bucket.unwrap_or(&config.r2.default_bucket);

    let r2_client = R2Client::new(
        config.cloudflare.endpoint.clone(),
        access_key_id,
        secret_access_key,
        bucket.to_string(),
    ).await?;

    match action {
        "upload" => {
            let file = file.ok_or_else(|| anyhow::anyhow!("Source file required"))?;
            let key = key.ok_or_else(|| anyhow::anyhow!("R2 key required"))?;

            let path = Path::new(file);
            if !path.exists() {
                return Err(anyhow::anyhow!("File not found: {}", file));
            }

            let file_size = path.metadata()?.len();

            println!("Uploading {} -> {}...", file, key);
            println!("  Size: {}", format_bytes(file_size as i64));

            // Detect content type
            let content_type = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            r2_client.upload_file(key, path, &content_type).await?;

            println!("  ✅ Upload complete");

            Ok(())
        }
        "download" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("R2 key required"))?;
            let dest = file.ok_or_else(|| anyhow::anyhow!("Destination required"))?;

            println!("Downloading {} -> {}...", key, dest);
            r2_client.download_file(key, Path::new(dest)).await?;
            println!("  ✅ Download complete");

            Ok(())
        }
        "delete" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("R2 key required"))?;

            println!("Deleting {}...", key);
            r2_client.delete_object(key).await?;
            println!("  ✅ File deleted");

            Ok(())
        }
        "ls" => {
            println!("Listing files (prefix: {:?})...", prefix);

            let objects = r2_client.list_objects(prefix).await?;

            if objects.is_empty() {
                println!("  No files found");
            } else {
                #[derive(Tabled)]
                struct ObjectRow {
                    key: String,
                    size: String,
                }

                let rows: Vec<ObjectRow> = objects.iter().map(|o| ObjectRow {
                    key: o.key.clone(),
                    size: format_bytes(o.size),
                }).collect();

                println!();
                println!("{}", Table::new(rows));
            }

            Ok(())
        }
        _ => {
            println!("Unknown action: {}", action);
            println!("Available actions: upload, download, delete, ls");
            Ok(())
        }
    }
}

/// Handle URLs commands
pub async fn handle_urls(action: &str, key: Option<&str>, expires: u64, output: &str) -> Result<()> {
    if action != "generate" {
        println!("Unknown action: {}", action);
        println!("Available actions: generate");
        return Ok(());
    }

    let key = key.ok_or_else(|| anyhow::anyhow!("R2 key required"))?;
    let config = load_config()?;

    // Get R2 credentials
    let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
        return Err(anyhow::anyhow!(
            "API Token not supported yet.\n\
             Use Access Keys in your configuration.\n\
             Get your Access Keys from: https://dash.cloudflare.com/{}/r2/api-tokens",
            config.cloudflare.account_id
        ));
    } else {
        (
            config.cloudflare.access_key_id.clone().unwrap(),
            config.cloudflare.secret_access_key.clone().unwrap(),
        )
    };

    let r2_client = R2Client::new(
        config.cloudflare.endpoint.clone(),
        access_key_id,
        secret_access_key,
        config.r2.default_bucket.clone(),
    ).await?;

    println!("Generating signed URL for {} (expires: {}s)...", key, expires);

    let url = r2_client.generate_presigned_url(
        key,
        std::time::Duration::from_secs(expires),
    ).await?;

    match output {
        "json" => {
            println!();
            println!("{}", serde_json::json!({
                "key": key,
                "url": url,
                "expires_in": expires,
                "expires_at": chrono::Utc::now() + chrono::Duration::seconds(expires as i64)
            }).to_string());
        }
        _ => {
            println!();
            println!("  ✅ URL generated:");
            println!("  {}", url);
            println!();
            println!("  Expires in: {}s", expires);
        }
    }

    Ok(())
}

/// Handle doctor commands
pub async fn handle_doctor(action: &str) -> Result<()> {
    match action {
        "check" => {
            println!("Checking r2pilot installation...");

            println!("  ✅ r2pilot is installed");
            println!("  Version: {}", env!("CARGO_PKG_VERSION"));

            // Check config
            let config_path = get_config_path()?;
            if config_path.exists() {
                println!("  ✅ Configuration found");

                let config = load_config()?;
                validate_config(&config)?;
                println!("  ✅ Configuration valid");
            } else {
                println!("  ⚠️  Configuration not found (run 'r2pilot init')");
            }

            Ok(())
        }
        "test-connection" => {
            println!("Testing R2 connection...");

            let config = load_config()?;

            // Get R2 credentials
            let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
                return Err(anyhow::anyhow!(
                    "API Token not supported yet.\n\
                     Use Access Keys in your configuration."
                ));
            } else {
                println!("  Using configured Access Keys");
                (
                    config.cloudflare.access_key_id.clone().unwrap(),
                    config.cloudflare.secret_access_key.clone().unwrap(),
                )
            };

            println!("  Testing R2 connection...");
            let r2_client = R2Client::new(
                config.cloudflare.endpoint.clone(),
                access_key_id,
                secret_access_key,
                config.r2.default_bucket.clone(),
            ).await?;

            let _objects = r2_client.list_objects(None).await?;
            println!("  ✅ R2 connection OK");

            println!();
            println!("  ✅ All connections are working!");

            Ok(())
        }
        _ => {
            println!("Unknown action: {}", action);
            println!("Available actions: check, test-connection");
            Ok(())
        }
    }
}

/// Format bytes to human-readable size
fn format_bytes(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Handle shell completion generation
pub async fn handle_completion(shell: &str, cmd: &mut Command) -> Result<()> {
    use std::io;

    let clap_shell = match shell {
        "bash" => ClapShell::Bash,
        "zsh" => ClapShell::Zsh,
        "fish" => ClapShell::Fish,
        "elvish" => ClapShell::Elvish,
        "powershell" | "pwsh" => ClapShell::PowerShell,
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported shell: {}\nSupported shells: bash, zsh, fish, elvish, powershell",
                shell
            ));
        }
    };

    println!("Generating completion for {:?}...", clap_shell);
    println!();

    // Generate completion script
    generate(clap_shell, cmd, "r2pilot", &mut io::stdout());

    println!();
    println!("✅ Completion generated!");
    println!();
    println!("Installation instructions:");

    match shell {
        "bash" => {
            println!("  # Add to your ~/.bashrc:");
            println!("  source <(r2pilot completion bash)");
            println!();
            println!("  # Or for permanent installation:");
            println!("  r2pilot completion bash > ~/.local/share/bash-completion/completions/r2pilot");
        }
        "zsh" => {
            println!("  # Add to your ~/.zshrc:");
            println!("  source <(r2pilot completion zsh)");
            println!();
            println!("  # Or for permanent installation:");
            println!("  r2pilot completion zsh > ~/.zsh/completion/_r2pilot");
            println!("  # then add to ~/.zshrc:");
            println!("  fpath=(~/.zsh/completion $fpath)");
            println!("  autoload -U compinit && compinit");
        }
        "fish" => {
            println!("  # Add to ~/.config/fish/completions/r2pilot.fish:");
            println!("  r2pilot completion fish > ~/.config/fish/completions/r2pilot.fish");
        }
        "elvish" => {
            println!("  # Add to ~/.elvish/rc.elv:");
            println!("  r2pilot completion elvish > ~/.elvish/lib/r2pilot.elv");
            println!("  # then add to rc.elv:");
            println!("  use ~/.elvish/lib/r2pilot");
        }
        "powershell" | "pwsh" => {
            println!("  # Run in PowerShell:");
            println!("  r2pilot completion powershell | Out-String | Invoke-Expression");
            println!();
            println!("  # Or add to your PowerShell Profile:");
            println!("  r2pilot completion powershell > $PROFILE");
        }
        _ => {}
    }

    Ok(())
}
