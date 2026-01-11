//! Command handlers for r2pilot CLI

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
pub async fn handle_tokens(_action: &str, _token_id: Option<&str>) -> Result<()> {
    println!("⚠️  Gestion des tokens via API non disponible");
    println!("   Gérez vos tokens depuis le dashboard Cloudflare:");
    println!("   https://dash.cloudflare.com/profile/api-tokens");
    println!();
    println!("   Pour R2, créez des API Tokens depuis:");
    println!("   https://dash.cloudflare.com/{}/r2/api-tokens", "<account_id>");

    Ok(())
}

/// Handle buckets commands
pub async fn handle_buckets(action: &str, name: Option<&str>) -> Result<()> {
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

    let bucket = name.unwrap_or(&config.r2.default_bucket);

    let r2_client = R2Client::new(
        config.cloudflare.endpoint.clone(),
        access_key_id,
        secret_access_key,
        bucket.to_string(),
    ).await?;

    match action {
        "list" => {
            println!("⚠️  Liste des buckets non disponible");
            println!("   Utilisez le dashboard Cloudflare: https://dash.cloudflare.com/");
            println!();
            println!("Bucket configuré: {}", config.r2.default_bucket);

            Ok(())
        }
        "create" => {
            println!("⚠️  Création de bucket non disponible");
            println!("   Créez le bucket depuis le dashboard Cloudflare:");
            println!("   https://dash.cloudflare.com/{}/r2/buckets", config.cloudflare.account_id);

            Ok(())
        }
        "delete" => {
            println!("⚠️  Suppression de bucket non disponible");
            println!("   Supprimez le bucket depuis le dashboard Cloudflare:");
            println!("   https://dash.cloudflare.com/{}/r2/buckets", config.cloudflare.account_id);

            Ok(())
        }
        "info" => {
            println!("Informations sur le bucket '{}'...", bucket);

            // Try to list objects as a connection test
            let objects = r2_client.list_objects(None).await?;

            println!("  Name: {}", bucket);
            println!("  Objects: {}", objects.len());

            Ok(())
        }
        "ls" => {
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

            Ok(())
        }
        _ => {
            println!("Action inconnue: {}", action);
            println!("Actions disponibles: list, create, delete, info, ls");
            Ok(())
        }
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
pub async fn handle_urls(action: &str, key: Option<&str>, expires: u64, _output: &str) -> Result<()> {
    if action != "generate" {
        println!("Action inconnue: {}", action);
        println!("Actions disponibles: generate");
        return Ok(());
    }

    let _key = key.ok_or_else(|| anyhow::anyhow!("Clé R2 requise"))?;

    println!("⚠️  Génération d'URLs signées non disponible");
    println!("   Utilisez le dashboard Cloudflare ou le presigned URL SDK");
    println!();
    println!("   Expiration: {}s", expires);

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
