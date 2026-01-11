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
            println!("Configuration actuelle:");
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
            println!("  Bucket par défaut: {}", config.r2.default_bucket);
            println!("  Region: {}", config.r2.region);
            println!("  Expiration par défaut: {}s", config.r2.default_expiration);

            Ok(())
        }
        "validate" => {
            println!("Validation des credentials...");

            let config = load_config()?;

            // Validate config format
            validate_config(&config)?;
            println!("  ✅ Format de configuration valide");

            // Get R2 credentials
            let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
                return Err(anyhow::anyhow!(
                    "Validation via API Token non encore implémenté.\n\
                     Utilisez les Access Keys dans votre configuration.\n\
                     Obtenez vos Access Keys depuis: https://dash.cloudflare.com/{}/r2/api-tokens",
                    config.cloudflare.account_id
                ));
            } else {
                (
                    config.cloudflare.access_key_id.clone()
                        .ok_or_else(|| anyhow::anyhow!("Access Key ID non configuré (lancez 'r2pilot init')"))?,
                    config.cloudflare.secret_access_key.clone()
                        .ok_or_else(|| anyhow::anyhow!("Secret Access Key non configuré (lancez 'r2pilot init')"))?,
                )
            };

            println!("  Test de connexion R2...");
            let r2_client = R2Client::new(
                config.cloudflare.endpoint.clone(),
                access_key_id,
                secret_access_key,
                config.r2.default_bucket.clone(),
            ).await?;

            // Try to list objects as a connection test
            let _objects = r2_client.list_objects(None).await?;

            println!("  ✅ Configuration valide !");
            println!("  ✅ Connexion R2 réussie !");

            Ok(())
        }
        "edit" => {
            println!("Ouverture de l'éditeur...");
            println!("  Fichier: ~/.config/r2pilot/config.toml");
            println!();

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let config_path = get_config_path()?;

            let status = std::process::Command::new(editor)
                .arg(&config_path)
                .status()?;

            if status.success() {
                println!("  ✅ Configuration éditée");

                // Validate after edit
                let config = load_config()?;
                validate_config(&config)?;
                println!("  ✅ Configuration valide");
            } else {
                println!("  ⚠️  Éditeur quitté avec erreur");
            }

            Ok(())
        }
        _ => {
            println!("Action inconnue: {}", action);
            println!("Actions disponibles: show, edit, validate");
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
        "API Token requis pour gérer les tokens.\n\
         Ajoutez 'api_token' dans votre configuration.\n\
         Obtenez un API Token depuis: https://dash.cloudflare.com/profile/api-tokens"
    ))?;

    let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());

    match action {
        "list" => {
            println!("Liste des API Tokens Cloudflare...");
            println!();

            let tokens = cf_client.list_tokens().await?;

            if tokens.is_empty() {
                println!("  Aucun token trouvé");
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
                .with_prompt("Nom du token")
                .default(format!("r2pilot-{}", chrono::Utc::now().format("%Y%m%d")))
                .interact()?;

            println!();
            println!("Création du token '{}'...", name);

            let builder = R2TokenBuilder::new(name.clone(), config.cloudflare.account_id.clone());
            let params = builder.build();

            let token = cf_client.create_token(params).await?;

            println!("  ✅ Token créé: {}", token.name);
            println!();
            println!("  IMPORTANT: Copiez ce token maintenant, il ne sera plus affiché !");
            println!("  Status: {}", format_status(&token.status));
            println!();
            println!("  ⚠️  Sauvegardez ce token dans votre configuration:");
            println!("     api_token = \"<votre_token>\"");

            Ok(())
        }
        "revoke" => {
            let id = token_id.ok_or_else(|| anyhow::anyhow!("Token ID requis (utilisez 'tokens list' pour voir les IDs)"))?;

            println!("⚠️  Attention: vous allez révoquer le token '{}'", id);
            println!("  Cette action est IRRÉVERSIBLE !");

            cf_client.revoke_token(id).await?;

            println!("  ✅ Token révoqué: {}", id);

            Ok(())
        }
        _ => {
            println!("Action inconnue: {}", action);
            println!("Actions disponibles: list, create, revoke");
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
                "API Token requis pour lister les buckets.\n\
                 Ajoutez 'api_token' dans votre configuration.\n\
                 Obtenez un API Token depuis: https://dash.cloudflare.com/{}/r2/api-tokens",
                config.cloudflare.account_id
            ))?;

            println!("Liste des buckets R2...");
            println!();

            let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());
            let buckets = cf_client.list_buckets().await?;

            if buckets.is_empty() {
                println!("  Aucun bucket trouvé");
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
            println!("Bucket par défaut: {}", config.r2.default_bucket);

            Ok(())
        }
        "create" => {
            let bucket_name = name.ok_or_else(|| anyhow::anyhow!("Nom du bucket requis"))?;

            let api_token = config.cloudflare.api_token.clone().ok_or_else(|| anyhow::anyhow!(
                "API Token requis pour créer des buckets.\n\
                 Ajoutez 'api_token' dans votre configuration.\n\
                 Obtenez un API Token depuis: https://dash.cloudflare.com/{}/r2/api-tokens",
                config.cloudflare.account_id
            ))?;

            println!("Création du bucket '{}'...", bucket_name);

            let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());
            let bucket = cf_client.create_bucket(bucket_name, "eu").await?;

            println!("  ✅ Bucket créé: {}", bucket.name);
            println!("  Location: {}", bucket.location);

            Ok(())
        }
        "delete" => {
            let bucket_name = name.ok_or_else(|| anyhow::anyhow!("Nom du bucket requis"))?;

            // Prevent accidental deletion of default bucket
            if bucket_name == config.r2.default_bucket {
                return Err(anyhow::anyhow!(
                    "Impossible de supprimer le bucket par défaut '{}'.\n\
                     Changez le bucket par défaut dans votre configuration d'abord.",
                    bucket_name
                ));
            }

            let api_token = config.cloudflare.api_token.clone().ok_or_else(|| anyhow::anyhow!(
                "API Token requis pour supprimer des buckets.\n\
                 Ajoutez 'api_token' dans votre configuration."
            ))?;

            println!("⚠️  Attention: vous allez supprimer le bucket '{}'", bucket_name);
            println!("  Cette action est IRRÉVERSIBLE !");

            let cf_client = CloudflareClient::new(api_token, config.cloudflare.account_id.clone());
            cf_client.delete_bucket(bucket_name).await?;

            println!("  ✅ Bucket supprimé: {}", bucket_name);

            Ok(())
        }
        "info" | "ls" => {
            // Get R2 credentials for S3 API access
            let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
                return Err(anyhow::anyhow!(
                    "La commande '{}' nécessite les Access Keys R2.\n\
                     Configurez access_key_id et secret_access_key pour utiliser les opérations S3.",
                    action
                ));
            } else {
                (
                    config.cloudflare.access_key_id.clone()
                        .ok_or_else(|| anyhow::anyhow!("Access Key ID non configuré (lancez 'r2pilot init')"))?,
                    config.cloudflare.secret_access_key.clone()
                        .ok_or_else(|| anyhow::anyhow!("Secret Access Key non configuré (lancez 'r2pilot init')"))?,
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
                println!("Informations sur le bucket '{}'...", bucket);

                // Try to list objects as a connection test
                let objects = r2_client.list_objects(None).await?;

                println!("  Name: {}", bucket);
                println!("  Objects: {}", objects.len());
            } else {
                println!("Contenu du bucket '{}'...", bucket);

                let objects = r2_client.list_objects(None).await?;

                if objects.is_empty() {
                    println!("  Bucket vide");
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
            println!("Action inconnue: {}", action);
            println!("Actions disponibles: list, create, delete, info, ls");
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
            "API Token non supporté pour l'instant.\n\
             Utilisez les Access Keys dans votre configuration.\n\
             Obtenez vos Access Keys depuis: https://dash.cloudflare.com/{}/r2/api-tokens",
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
            let file = file.ok_or_else(|| anyhow::anyhow!("Fichier source requis"))?;
            let key = key.ok_or_else(|| anyhow::anyhow!("Clé R2 requise"))?;

            let path = Path::new(file);
            if !path.exists() {
                return Err(anyhow::anyhow!("Fichier non trouvé: {}", file));
            }

            let file_size = path.metadata()?.len();

            println!("Upload de {} -> {}...", file, key);
            println!("  Taille: {}", format_bytes(file_size as i64));

            // Detect content type
            let content_type = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            r2_client.upload_file(key, path, &content_type).await?;

            println!("  ✅ Upload terminé");

            Ok(())
        }
        "download" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Clé R2 requise"))?;
            let dest = file.ok_or_else(|| anyhow::anyhow!("Destination requise"))?;

            println!("Download de {} -> {}...", key, dest);
            r2_client.download_file(key, Path::new(dest)).await?;
            println!("  ✅ Download terminé");

            Ok(())
        }
        "delete" => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Clé R2 requise"))?;

            println!("Suppression de {}...", key);
            r2_client.delete_object(key).await?;
            println!("  ✅ Fichier supprimé");

            Ok(())
        }
        "ls" => {
            println!("Liste des fichiers (prefix: {:?})...", prefix);

            let objects = r2_client.list_objects(prefix).await?;

            if objects.is_empty() {
                println!("  Aucun fichier trouvé");
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
            println!("Action inconnue: {}", action);
            println!("Actions disponibles: upload, download, delete, ls");
            Ok(())
        }
    }
}

/// Handle URLs commands
pub async fn handle_urls(action: &str, key: Option<&str>, expires: u64, output: &str) -> Result<()> {
    if action != "generate" {
        println!("Action inconnue: {}", action);
        println!("Actions disponibles: generate");
        return Ok(());
    }

    let key = key.ok_or_else(|| anyhow::anyhow!("Clé R2 requise"))?;
    let config = load_config()?;

    // Get R2 credentials
    let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
        return Err(anyhow::anyhow!(
            "API Token non supporté pour l'instant.\n\
             Utilisez les Access Keys dans votre configuration.\n\
             Obtenez vos Access Keys depuis: https://dash.cloudflare.com/{}/r2/api-tokens",
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

    println!("Génération URL signée pour {} (expires: {}s)...", key, expires);

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
            println!("  ✅ URL générée:");
            println!("  {}", url);
            println!();
            println!("  Expire dans: {}s", expires);
        }
    }

    Ok(())
}

/// Handle doctor commands
pub async fn handle_doctor(action: &str) -> Result<()> {
    match action {
        "check" => {
            println!("Vérification de l'installation r2pilot...");

            println!("  ✅ r2pilot est installé");
            println!("  Version: {}", env!("CARGO_PKG_VERSION"));

            // Check config
            let config_path = get_config_path()?;
            if config_path.exists() {
                println!("  ✅ Configuration trouvée");

                let config = load_config()?;
                validate_config(&config)?;
                println!("  ✅ Configuration valide");
            } else {
                println!("  ⚠️  Configuration non trouvée (lancez 'r2pilot init')");
            }

            Ok(())
        }
        "test-connection" => {
            println!("Test de connexion R2...");

            let config = load_config()?;

            // Get R2 credentials
            let (access_key_id, secret_access_key) = if let Some(_token) = &config.cloudflare.api_token {
                return Err(anyhow::anyhow!(
                    "API Token non supporté pour l'instant.\n\
                     Utilisez les Access Keys dans votre configuration."
                ));
            } else {
                println!("  Utilisation des Access Keys configurés");
                (
                    config.cloudflare.access_key_id.clone().unwrap(),
                    config.cloudflare.secret_access_key.clone().unwrap(),
                )
            };

            println!("  Test de connexion R2...");
            let r2_client = R2Client::new(
                config.cloudflare.endpoint.clone(),
                access_key_id,
                secret_access_key,
                config.r2.default_bucket.clone(),
            ).await?;

            let _objects = r2_client.list_objects(None).await?;
            println!("  ✅ Connexion R2 OK");

            println!();
            println!("  ✅ Toutes les connexions sont fonctionnelles !");

            Ok(())
        }
        _ => {
            println!("Action inconnue: {}", action);
            println!("Actions disponibles: check, test-connection");
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
                "Shell non supporté: {}\nShells supportés: bash, zsh, fish, elvish, powershell",
                shell
            ));
        }
    };

    println!("Génération de la completion pour {:?}...", clap_shell);
    println!();

    // Generate completion script
    generate(clap_shell, cmd, "r2pilot", &mut io::stdout());

    println!();
    println!("✅ Completion générée !");
    println!();
    println!("Instructions d'installation:");

    match shell {
        "bash" => {
            println!("  # Ajoutez à votre ~/.bashrc:");
            println!("  source <(r2pilot completion bash)");
            println!();
            println!("  # Ou pour une installation permanente:");
            println!("  r2pilot completion bash > ~/.local/share/bash-completion/completions/r2pilot");
        }
        "zsh" => {
            println!("  # Ajoutez à votre ~/.zshrc:");
            println!("  source <(r2pilot completion zsh)");
            println!();
            println!("  # Ou pour une installation permanente:");
            println!("  r2pilot completion zsh > ~/.zsh/completion/_r2pilot");
            println!("  # puis ajoutez à ~/.zshrc:");
            println!("  fpath=(~/.zsh/completion $fpath)");
            println!("  autoload -U compinit && compinit");
        }
        "fish" => {
            println!("  # Ajoutez à votre ~/.config/fish/completions/r2pilot.fish:");
            println!("  r2pilot completion fish > ~/.config/fish/completions/r2pilot.fish");
        }
        "elvish" => {
            println!("  # Ajoutez à votre ~/.elvish/rc.elv:");
            println!("  r2pilot completion elvish > ~/.elvish/lib/r2pilot.elv");
            println!("  # puis ajoutez à votre rc.elv:");
            println!("  use ~/.elvish/lib/r2pilot");
        }
        "powershell" | "pwsh" => {
            println!("  # Exécutez dans PowerShell:");
            println!("  r2pilot completion powershell | Out-String | Invoke-Expression");
            println!();
            println!("  # Ou ajoutez à votre PowerShell Profile:");
            println!("  r2pilot completion powershell > $PROFILE");
        }
        _ => {}
    }

    Ok(())
}
