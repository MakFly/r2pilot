# r2pilot Documentation

> [Français](../fr/README.md) | English

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Configuration](#configuration)
- [Commands](#commands)
- [Examples](#examples)

## Overview

**r2pilot** is a Rust CLI tool to manage Cloudflare R2 storage from your terminal.

### Features

- **Bucket Management**: List, create, delete buckets
- **File Operations**: Upload, download, delete files with automatic multipart support
- **Signed URLs**: Generate presigned URLs for GET, PUT, DELETE
- **CORS**: CORS configuration (interactive or JSON)
- **Lifecycle Rules**: Lifecycle rules for automatic deletion
- **Website/Hosting**: Static hosting configuration (public bucket)
- **Interactive Setup**: Guided configuration wizard
- **Multiple Authentication**: Support for API Tokens and Access Keys

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/MakFly/r2pilot.git
cd r2pilot

# Build release
cargo build --release

# Create symlink
ln -s $(pwd)/target/release/r2pilot ~/bin/r2pilot

# Reload your shell
source ~/.zshrc  # or source ~/.bashrc
```

### Requirements

- Rust 1.70+ (for building from source)
- Cloudflare Account with R2 enabled

## Configuration

### Initial Setup

Run the interactive wizard:

```bash
r2pilot init
```

You'll need:
1. Your Cloudflare Account ID (32 characters, alphanumeric)
2. Authentication method:
   - **API Token** (recommended for Cloudflare API operations)
   - **Access Key ID + Secret Access Key** (required for S3-compatible operations)
3. Your default R2 bucket name

### Configuration File

The configuration is stored in `~/.config/r2pilot/config.toml`:

```toml
[cloudflare]
account_id = "your_account_id"
endpoint = "https://your_account_id.r2.cloudflarestorage.com"
api_token = "your_api_token"  # OR access_key_id + secret_access_key
access_key_id = "your_access_key_id"
secret_access_key = "your_secret_access_key"

[r2]
default_bucket = "your_bucket_name"
region = "auto"
default_expiration = 7200  # 2 hours in seconds
```

### Getting Your Credentials

**API Token** (for bucket management):
1. Go to https://dash.cloudflare.com/profile/api-tokens
2. Create a token with R2 Edit permissions

**Access Keys** (for file operations):
1. Go to https://dash.cloudflare.com/<account_id>/r2/api-tokens
2. Manage R2 API Tokens for your bucket
3. Get your Access Key ID and Secret Access Key

## Commands

### init

Initialize configuration with interactive wizard.

```bash
r2pilot init
```

### config

Manage configuration.

```bash
# Show current configuration
r2pilot config show

# Edit configuration in $EDITOR
r2pilot config edit

# Validate credentials and test connection
r2pilot config validate
```

### tokens

Manage Cloudflare API tokens.

```bash
# List all API tokens
r2pilot tokens list

# Create a new R2 token
r2pilot tokens create

# Revoke a token
r2pilot tokens revoke <token_id>
```

### buckets

Manage R2 buckets.

```bash
# List all buckets
r2pilot buckets list

# Create a new bucket
r2pilot buckets create my-bucket

# Delete a bucket
r2pilot buckets delete my-bucket

# Get bucket information
r2pilot buckets info my-bucket

# List bucket contents
r2pilot buckets ls my-bucket
```

### files

Manage files in R2.

```bash
# Upload a file
r2pilot files upload local-file.txt path/to/remote.txt --bucket my-bucket --progress

# Upload large file with multipart (automatic >100MB)
r2pilot files upload largefile.iso backups/large.iso --progress

# Force multipart upload
r2pilot files upload file.txt path/to/file.txt --multipart

# Download a file
r2pilot files download path/to/remote.txt local-file.txt --bucket my-bucket

# Delete a file
r2pilot files delete path/to/remote.txt --bucket my-bucket

# List files
r2pilot files ls --prefix path/to/
```

### urls

Generate signed URLs.

```bash
# Generate a signed GET URL (default: 2 hours)
r2pilot urls generate path/to/file.txt

# Custom method (GET, PUT, DELETE)
r2pilot urls generate path/to/file.txt --method put --expires 3600 --content-type video/mp4

# Custom expiration (in seconds)
r2pilot urls generate path/to/file.txt --expires 3600

# JSON output
r2pilot urls generate path/to/file.txt --output json
```

### cors

Manage bucket CORS configuration.

```bash
# View CORS configuration
r2pilot cors get

# Configure CORS in interactive mode
r2pilot cors set --interactive

# Configure CORS via JSON file
r2pilot cors set --file cors.json

# Delete CORS configuration
r2pilot cors delete
```

**Example CORS JSON file:**
```json
{
  "rules": [
    {
      "allowedOrigins": ["https://example.com", "https://app.example.com"],
      "allowedMethods": ["GET", "PUT", "DELETE", "HEAD"],
      "allowedHeaders": ["*"],
      "maxAgeSeconds": 86400
    }
  ]
}
```

### lifecycle

Manage object lifecycle rules.

```bash
# View lifecycle rules
r2pilot lifecycle get

# Configure rules in interactive mode
r2pilot lifecycle set --interactive

# Configure via JSON file
r2pilot lifecycle set --file lifecycle.json

# Delete lifecycle rules
r2pilot lifecycle delete
```

**Example Lifecycle JSON file:**
```json
{
  "rules": [
    {
      "id": "delete-old-videos",
      "filter": {
        "prefix": "videos/"
      },
      "status": "Enabled",
      "expiration": {
        "days": 30
      }
    }
  ]
}
```

### website

Manage static hosting (public bucket).

```bash
# Enable static hosting
r2pilot website enable --index index.html --error 404.html

# View website configuration
r2pilot website get

# Disable static hosting
r2pilot website disable
```

**Note:** Once enabled, your bucket will be publicly accessible at:
`https://<bucket_name>.<account_id>.r2.cloudflarestorage.com/<file_path>`

### completion

Generate shell completion scripts.

```bash
# Generate for bash
r2pilot completion bash

# Generate for zsh
r2pilot completion zsh

# Generate for fish
r2pilot completion fish
```

### doctor

Diagnostics and troubleshooting.

```bash
# Check installation
r2pilot doctor check

# Test R2 connection
r2pilot doctor test-connection
```

## Examples

### First Time Setup

```bash
# Run the setup wizard
r2pilot init

# Validate your configuration
r2pilot config validate
```

### Upload a Video

```bash
# Upload to R2 with progress bar
r2pilot files upload video.mp4 raw/video-2024-01.mp4 --progress

# Upload to specific bucket
r2pilot files upload video.mp4 videos/january.mp4 --bucket my-videos

# Upload large file (>100MB) with automatic multipart
r2pilot files upload largefile.iso backups/large.iso --progress
```

### Generate Shareable Link

```bash
# Generate link valid for 2 hours (default)
r2pilot urls generate videos/january.mp4

# Generate link valid for 1 hour
r2pilot urls generate videos/january.mp4 --expires 3600

# Generate a PUT URL for direct browser upload
r2pilot urls generate uploads/video.mp4 --method put --expires 3600 --content-type video/mp4

# Get JSON output for scripts
r2pilot urls generate videos/january.mp4 --output json
```

### Configure CORS for a Web Application

```bash
# Interactive mode
r2pilot cors set --interactive

# Via JSON file
r2pilot cors set --file cors.json

# Verify configuration
r2pilot cors get
```

### Configure Automatic Deletion (Lifecycle)

```bash
# Interactive mode - delete videos older than 30 days
r2pilot lifecycle set --interactive

# Via JSON file
r2pilot lifecycle set --file lifecycle.json

# Verify rules
r2pilot lifecycle get
```

### Host a Static Website

```bash
# Enable static hosting
r2pilot website enable --index index.html --error 404.html

# Upload your website files
r2pilot files upload index.html index.html
r2pilot files upload styles.css styles.css

# Your site is now live!
# URL: https://<bucket>.<account_id>.r2.cloudflarestorage.com/index.html
```

### List and Manage Buckets

```bash
# List all buckets
r2pilot buckets list

# List bucket contents
r2pilot buckets ls my-bucket

# Create new bucket
r2pilot buckets create backups

# Delete bucket (not the default one)
r2pilot buckets delete old-bucket
```

### Troubleshooting

```bash
# Check if r2pilot is installed correctly
r2pilot doctor check

# Test R2 connection
r2pilot doctor test-connection

# Show current configuration
r2pilot config show
```

## Tips

- **Default Bucket**: Set a default bucket to avoid specifying `--bucket` every time
- **Progress Bar**: Use `--progress` flag for large file uploads
- **JSON Output**: Use `--output json` for scripting and automation
- **Shell Completion**: Enable completion for better command experience

## Troubleshooting

### "Access Key ID not configured"

Run `r2pilot init` to configure your credentials, or manually edit `~/.config/r2pilot/config.toml`.

### "API Token required for this operation"

Some operations require an API Token with R2 permissions:
- Buckets management (list, create, delete)
- Token management

Get your API Token from: https://dash.cloudflare.com/profile/api-tokens

### "Access Keys required for this operation"

File operations (upload, download) require R2 Access Keys:
1. Go to your R2 dashboard
2. Navigate to API Tokens
3. Get your Access Key ID and Secret Access Key

## License

MIT - See [LICENSE](../../LICENSE) for details.
