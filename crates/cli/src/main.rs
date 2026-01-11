use anyhow::Result;
use clap::{Parser, CommandFactory};
use color_eyre::config::HookBuilder;

mod handlers;
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
        Commands::Init => handlers::handle_init().await,
        Commands::Config { action } => {
            let action_str = match action {
                ConfigAction::Show => "show",
                ConfigAction::Edit => "edit",
                ConfigAction::Validate => "validate",
            };
            handlers::handle_config(action_str).await
        }
        Commands::Tokens { action } => {
            let (action_str, token_id) = match action {
                TokenAction::List => ("list", None),
                TokenAction::Create => ("create", None),
                TokenAction::Revoke { token_id } => ("revoke", Some(token_id)),
            };
            handlers::handle_tokens(action_str, token_id.as_deref()).await
        }
        Commands::Buckets { action } => {
            let (action_str, name) = match action {
                BucketAction::List => ("list", None),
                BucketAction::Create { name } => ("create", Some(name)),
                BucketAction::Delete { name } => ("delete", Some(name)),
                BucketAction::Info { name } => ("info", Some(name)),
                BucketAction::Ls { name } => ("ls", name),
            };
            handlers::handle_buckets(action_str, name.as_deref()).await
        }
        Commands::Files { action } => {
            let (action_str, file, key, bucket, prefix, progress) = match action {
                FileAction::Upload { file, key, bucket, progress } => ("upload", Some(file), Some(key), bucket, None, progress),
                FileAction::Download { key, dest, bucket } => ("download", Some(dest), Some(key), bucket, None, false),
                FileAction::Delete { key, bucket } => ("delete", None, Some(key), bucket, None, false),
                FileAction::Ls { prefix, bucket } => ("ls", None, None, bucket, prefix, false),
            };
            handlers::handle_files(action_str, file.as_deref(), key.as_deref(), bucket.as_deref(), prefix.as_deref(), progress).await
        }
        Commands::Urls { action } => {
            let (action_str, key, expires, output) = match action {
                UrlAction::Generate { key, expires, output } => ("generate", Some(key), expires, output),
            };
            handlers::handle_urls(action_str, key.as_deref(), expires, &output).await
        }
        Commands::Completion { shell } => {
            handlers::handle_completion(&shell, &mut Cli::command()).await
        }
        Commands::Doctor { action } => {
            let action_str = match action {
                DoctorAction::Check => "check",
                DoctorAction::TestConnection => "test-connection",
            };
            handlers::handle_doctor(action_str).await
        }
    }
}
