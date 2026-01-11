use anyhow::Result;
use clap::{Parser, CommandFactory};
use color_eyre::config::HookBuilder;

mod handlers;
mod wizard;

/// r2pilot - CLI to manage Cloudflare R2
#[derive(Parser, Debug)]
#[command(name = "r2pilot")]
#[command(author = "Kev <kev@m7academy.com>")]
#[command(version = "0.1.0")]
#[command(about = "Rust CLI to manage Cloudflare R2 from your terminal", long_about = None)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Initial configuration (interactive wizard)
    Init,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Cloudflare API tokens management
    Tokens {
        #[command(subcommand)]
        action: TokenAction,
    },

    /// R2 buckets management
    Buckets {
        #[command(subcommand)]
        action: BucketAction,
    },

    /// File management
    Files {
        #[command(subcommand)]
        action: FileAction,
    },

    /// Signed URLs generation
    Urls {
        #[command(subcommand)]
        action: UrlAction,
    },

    /// Shell completion
    Completion {
        /// Shell type (bash, zsh, fish, elvish, powershell)
        shell: String,
    },

    /// Diagnostics and verification
    Doctor {
        #[command(subcommand)]
        action: DoctorAction,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration in $EDITOR
    Edit,
    /// Validate credentials
    Validate,
}

#[derive(clap::Subcommand, Debug)]
enum TokenAction {
    /// List API tokens
    List,
    /// Create a new token
    Create,
    /// Revoke a token
    Revoke { token_id: String },
}

#[derive(clap::Subcommand, Debug)]
enum BucketAction {
    /// List buckets
    List,
    /// Create a bucket
    Create { name: String },
    /// Delete a bucket
    Delete { name: String },
    /// Bucket information
    Info { name: String },
    /// List bucket contents
    Ls { name: Option<String> },
}

#[derive(clap::Subcommand, Debug)]
enum FileAction {
    /// Upload a file
    Upload {
        /// Local file to upload
        file: String,
        /// R2 key (destination)
        key: String,
        /// Target bucket (uses default bucket)
        #[arg(short, long)]
        bucket: Option<String>,
        /// Show progress bar
        #[arg(short, long)]
        progress: bool,
    },
    /// Download a file
    Download {
        /// R2 key
        key: String,
        /// Local destination
        dest: String,
        /// Source bucket (uses default bucket)
        #[arg(short, long)]
        bucket: Option<String>,
    },
    /// Delete a file
    Delete {
        /// R2 key
        key: String,
        /// Target bucket (uses default bucket)
        #[arg(short, long)]
        bucket: Option<String>,
    },
    /// List files
    Ls {
        /// Prefix to filter results
        prefix: Option<String>,
        /// Target bucket (uses default bucket)
        #[arg(short, long)]
        bucket: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
enum UrlAction {
    /// Generate a signed URL
    Generate {
        /// R2 key
        key: String,
        /// Expiration in seconds (default: 7200)
        #[arg(short, long, default_value = "7200")]
        expires: u64,
        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        output: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum DoctorAction {
    /// Check installation
    Check,
    /// Test R2 connection
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
