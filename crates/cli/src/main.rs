use anyhow::Result;
use clap::Parser;
use clap_complete::Shell as ClapShell;
use color_eyre::config::HookBuilder;
use std::str::FromStr;

mod wizard;

/// r2pilot - CLI pour gérer Cloudflare R2
#[derive(Parser, Debug)]
#[command(name = "r2pilot")]
#[command(author = "Kev <kev@m7academy.com>")]
#[command(version = "0.1.0")]
#[command(about = "CLI Rust pour gérer Cloudflare R2 depuis votre terminal", long_about = None)]
struct Cli {
    /// Sous-commande à exécuter
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Configuration initiale (wizard interactif)
    Init,

    /// Gestion de la configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Gestion des API tokens Cloudflare
    Tokens {
        #[command(subcommand)]
        action: TokenAction,
    },

    /// Gestion des buckets R2
    Buckets {
        #[command(subcommand)]
        action: BucketAction,
    },

    /// Gestion des fichiers
    Files {
        #[command(subcommand)]
        action: FileAction,
    },

    /// Génération d'URLs signées
    Urls {
        #[command(subcommand)]
        action: UrlAction,
    },

    /// Shell completion
    Completion {
        /// Shell type (bash, zsh, fish, elvish, powershell)
        shell: String,
    },

    /// Diagnostic et vérification
    Doctor {
        #[command(subcommand)]
        action: DoctorAction,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ConfigAction {
    /// Afficher la configuration actuelle
    Show,
    /// Éditer la configuration dans $EDITOR
    Edit,
    /// Valider les credentials
    Validate,
}

#[derive(clap::Subcommand, Debug)]
enum TokenAction {
    /// Lister les API tokens
    List,
    /// Créer un nouveau token
    Create,
    /// Révoquer un token
    Revoke { token_id: String },
}

#[derive(clap::Subcommand, Debug)]
enum BucketAction {
    /// Lister les buckets
    List,
    /// Créer un bucket
    Create { name: String },
    /// Supprimer un bucket
    Delete { name: String },
    /// Informations sur un bucket
    Info { name: String },
    /// Lister le contenu d'un bucket
    Ls { name: Option<String> },
}

#[derive(clap::Subcommand, Debug)]
enum FileAction {
    /// Uploader un fichier
    Upload {
        /// Fichier local à uploader
        file: String,
        /// Clé R2 (destination)
        key: String,
        /// Bucket cible ( utilise le bucket par défaut)
        #[arg(short, long)]
        bucket: Option<String>,
        /// Afficher la barre de progression
        #[arg(short, long)]
        progress: bool,
    },
    /// Télécharger un fichier
    Download {
        /// Clé R2
        key: String,
        /// Destination locale
        dest: String,
        /// Bucket source ( utilise le bucket par défaut)
        #[arg(short, long)]
        bucket: Option<String>,
    },
    /// Supprimer un fichier
    Delete {
        /// Clé R2
        key: String,
        /// Bucket cible ( utilise le bucket par défaut)
        #[arg(short, long)]
        bucket: Option<String>,
    },
    /// Lister les fichiers
    Ls {
        /// Préfixe pour filtrer les résultats
        prefix: Option<String>,
        /// Bucket cible ( utilise le bucket par défaut)
        #[arg(short, long)]
        bucket: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
enum UrlAction {
    /// Générer une URL signée
    Generate {
        /// Clé R2
        key: String,
        /// Expiration en secondes (défaut: 7200)
        #[arg(short, long, default_value = "7200")]
        expires: u64,
        /// Format de sortie (table, json)
        #[arg(short, long, default_value = "table")]
        output: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum DoctorAction {
    /// Vérifier l'installation
    Check,
    /// Tester la connexion R2
    TestConnection,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup error handling
    if let Err(e) = HookBuilder::default().install() {
        eprintln!("Warning: Failed to install error handler: {}", e);
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Commands::Init => {
            wizard::run_init_wizard().await
        }
        Commands::Config { action } => handle_config(action).await,
        Commands::Tokens { action } => handle_tokens(action).await,
        Commands::Buckets { action } => handle_buckets(action).await,
        Commands::Files { action } => handle_files(action).await,
        Commands::Urls { action } => handle_urls(action).await,
        Commands::Completion { shell } => {
            let clap_shell = match shell.as_str() {
                "bash" => ClapShell::Bash,
                "zsh" => ClapShell::Zsh,
                "fish" => ClapShell::Fish,
                "elvish" => ClapShell::Elvish,
                "powershell" => ClapShell::PowerShell,
                _ => {
                    println!("Shell non supporté: {}", shell);
                    println!("Shells supportés: bash, zsh, fish, elvish, powershell");
                    return Ok(());
                }
            };
            println!("Shell completion pour {:?}", clap_shell);
            // TODO: Generate completion scripts
            Ok(())
        }
        Commands::Doctor { action } => handle_doctor(action).await,
    }
}

async fn handle_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            println!("Configuration actuelle:");
            // TODO: Load and display config
            println!("  (Non configuré - lancez 'r2pilot init')");
        }
        ConfigAction::Edit => {
            println!("Ouverture de l'éditeur...");
            // TODO: Open config in $EDITOR
        }
        ConfigAction::Validate => {
            println!("Validation des credentials...");
            // TODO: Validate credentials
        }
    }
    Ok(())
}

async fn handle_tokens(action: TokenAction) -> Result<()> {
    match action {
        TokenAction::List => {
            println!("Liste des API tokens:");
            // TODO: List tokens
        }
        TokenAction::Create => {
            println!("Création d'un nouveau token...");
            // TODO: Create token
        }
        TokenAction::Revoke { token_id } => {
            println!("Révocation du token {}...", token_id);
            // TODO: Revoke token
        }
    }
    Ok(())
}

async fn handle_buckets(action: BucketAction) -> Result<()> {
    match action {
        BucketAction::List => {
            println!("Liste des buckets R2:");
            // TODO: List buckets
        }
        BucketAction::Create { name } => {
            println!("Création du bucket '{}'...", name);
            // TODO: Create bucket
        }
        BucketAction::Delete { name } => {
            println!("Suppression du bucket '{}'...", name);
            // TODO: Delete bucket
        }
        BucketAction::Info { name } => {
            println!("Informations sur le bucket '{}'...", name);
            // TODO: Get bucket info
        }
        BucketAction::Ls { name } => {
            println!("Contenu du bucket {:?}...", name);
            // TODO: List bucket contents
        }
    }
    Ok(())
}

async fn handle_files(action: FileAction) -> Result<()> {
    match action {
        FileAction::Upload { file, key, bucket, progress } => {
            println!("Upload de {} -> {}...", file, key);
            if progress {
                println!("  (avec progress bar)");
            }
            // TODO: Upload file
        }
        FileAction::Download { key, dest, bucket } => {
            println!("Download de {} -> {}...", key, dest);
            // TODO: Download file
        }
        FileAction::Delete { key, bucket } => {
            println!("Suppression de {}...", key);
            // TODO: Delete file
        }
        FileAction::Ls { prefix, bucket } => {
            println!("Liste des fichiers (prefix: {:?})...", prefix);
            // TODO: List files
        }
    }
    Ok(())
}

async fn handle_urls(action: UrlAction) -> Result<()> {
    match action {
        UrlAction::Generate { key, expires, output } => {
            println!("Génération URL signée pour {} (expires: {}s, output: {})...", key, expires, output);
            // TODO: Generate signed URL
        }
    }
    Ok(())
}

async fn handle_doctor(action: DoctorAction) -> Result<()> {
    match action {
        DoctorAction::Check => {
            println!("Vérification de l'installation r2pilot...");
            println!("  ✅ r2pilot est installé");
            // TODO: Check config, connection, etc.
        }
        DoctorAction::TestConnection => {
            println!("Test de connexion R2...");
            // TODO: Test R2 connection
        }
    }
    Ok(())
}
