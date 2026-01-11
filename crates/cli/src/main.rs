use anyhow::Result;
use clap::{CommandFactory, Parser};
use color_eyre::config::HookBuilder;

mod cors_wizard;
mod handlers;
mod lifecycle_wizard;
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

    /// CORS configuration management
    Cors {
        #[command(subcommand)]
        action: CorsAction,
    },

    /// Lifecycle rules management
    Lifecycle {
        #[command(subcommand)]
        action: LifecycleAction,
    },

    /// Public bucket settings (static hosting)
    Website {
        #[command(subcommand)]
        action: WebsiteAction,
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
        /// Force multipart upload
        #[arg(long)]
        multipart: bool,
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
        /// HTTP method (get, put, delete)
        #[arg(short, long, default_value = "get")]
        method: String,
        /// Expiration in seconds (default: 7200)
        #[arg(short, long, default_value = "7200")]
        expires: u64,
        /// Content type (for PUT requests)
        #[arg(long)]
        content_type: Option<String>,
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

#[derive(clap::Subcommand, Debug)]
enum CorsAction {
    /// Get CORS configuration
    Get { name: Option<String> },
    /// Set CORS configuration (interactive or JSON)
    Set {
        /// Bucket name
        #[arg(short, long)]
        bucket: Option<String>,
        /// JSON file with CORS rules
        #[arg(short, long)]
        file: Option<String>,
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    /// Delete CORS configuration
    Delete {
        /// Bucket name
        #[arg(short, long)]
        bucket: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
enum LifecycleAction {
    /// Get lifecycle rules
    Get { name: Option<String> },
    /// Set lifecycle rules (interactive or JSON)
    Set {
        /// Bucket name
        #[arg(short, long)]
        bucket: Option<String>,
        /// JSON file with lifecycle rules
        #[arg(short, long)]
        file: Option<String>,
        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    /// Delete lifecycle rules
    Delete {
        /// Bucket name
        #[arg(short, long)]
        bucket: Option<String>,
    },
}

#[derive(clap::Subcommand, Debug)]
enum WebsiteAction {
    /// Enable static hosting
    Enable {
        /// Bucket name
        #[arg(short, long)]
        bucket: Option<String>,
        /// Index document
        #[arg(long)]
        index: Option<String>,
        /// Error document
        #[arg(long)]
        error: Option<String>,
    },
    /// Disable static hosting
    Disable {
        /// Bucket name
        #[arg(short, long)]
        bucket: Option<String>,
    },
    /// Get website configuration
    Get { name: Option<String> },
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
            let (action_str, file, key, bucket, prefix, progress, multipart) = match action {
                FileAction::Upload {
                    file,
                    key,
                    bucket,
                    progress,
                    multipart,
                } => (
                    "upload",
                    Some(file),
                    Some(key),
                    bucket,
                    None,
                    progress,
                    multipart,
                ),
                FileAction::Download { key, dest, bucket } => (
                    "download",
                    Some(dest),
                    Some(key),
                    bucket,
                    None,
                    false,
                    false,
                ),
                FileAction::Delete { key, bucket } => {
                    ("delete", None, Some(key), bucket, None, false, false)
                }
                FileAction::Ls { prefix, bucket } => {
                    ("ls", None, None, bucket, prefix, false, false)
                }
            };
            handlers::handle_files(
                action_str,
                file.as_deref(),
                key.as_deref(),
                bucket.as_deref(),
                prefix.as_deref(),
                progress,
                multipart,
            )
            .await
        }
        Commands::Urls { action } => {
            let (action_str, key, method, expires, content_type_string, output) = match action {
                UrlAction::Generate {
                    key,
                    method,
                    expires,
                    content_type,
                    output,
                } => ("generate", Some(key), method, expires, content_type, output),
            };
            let content_type_ref = content_type_string.as_deref();
            handlers::handle_urls(
                action_str,
                key.as_deref(),
                &method,
                expires,
                content_type_ref,
                &output,
            )
            .await
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
        Commands::Cors { action } => {
            let (action_str, bucket, file, interactive) = match action {
                CorsAction::Get { name } => ("get", name, None, false),
                CorsAction::Set {
                    bucket,
                    file,
                    interactive,
                } => ("set", bucket, file, interactive),
                CorsAction::Delete { bucket } => ("delete", bucket, None, false),
            };
            handlers::handle_cors(action_str, bucket.as_deref(), file.as_deref(), interactive).await
        }
        Commands::Lifecycle { action } => {
            let (action_str, bucket, file, interactive) = match action {
                LifecycleAction::Get { name } => ("get", name, None, false),
                LifecycleAction::Set {
                    bucket,
                    file,
                    interactive,
                } => ("set", bucket, file, interactive),
                LifecycleAction::Delete { bucket } => ("delete", bucket, None, false),
            };
            handlers::handle_lifecycle(action_str, bucket.as_deref(), file.as_deref(), interactive)
                .await
        }
        Commands::Website { action } => {
            let (action_str, bucket, index, error) = match action {
                WebsiteAction::Enable {
                    bucket,
                    index,
                    error,
                } => ("enable", bucket, index, error),
                WebsiteAction::Disable { bucket } => ("disable", bucket, None, None),
                WebsiteAction::Get { name } => ("get", name, None, None),
            };
            handlers::handle_website(
                action_str,
                bucket.as_deref(),
                index.as_deref(),
                error.as_deref(),
            )
            .await
        }
    }
}
