//! R2 Client implementation using AWS S3 SDK

use crate::error::{Error, Result};
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// R2 client for managing Cloudflare R2 storage
pub struct R2Client {
    client: Client,
    bucket: String,
}

impl R2Client {
    /// Create a new R2 client
    pub async fn new(
        endpoint: String,
        access_key_id: String,
        secret_access_key: String,
        bucket: String,
    ) -> Result<Self> {
        // Create credentials
        let credentials = Credentials::new(&access_key_id, &secret_access_key, None, None, "r2pilot");

        // Build AWS config for R2 (S3-compatible)
        let config_builder = aws_sdk_s3::Config::builder()
            .endpoint_url(endpoint)
            .region(Region::new("auto".to_string()))
            .credentials_provider(credentials);

        let config = config_builder.build();

        let client = Client::from_conf(config);

        Ok(Self { client, bucket })
    }

    /// Upload a file to R2
    pub async fn upload_file(
        &self,
        key: &str,
        file_path: &Path,
        content_type: &str,
    ) -> Result<()> {
        // Read file content
        let mut file = File::open(file_path).await.map_err(|e| {
            Error::Io(e)
        })?;

        let metadata = file.metadata().await.map_err(|e| Error::Io(e))?;
        let buffer_size = metadata.len() as usize;
        let mut buffer = Vec::with_capacity(buffer_size);

        file.read_to_end(&mut buffer).await.map_err(|e| Error::Io(e))?;

        // Upload to R2
        self.upload_bytes(key, buffer, content_type).await
    }

    /// Upload bytes to R2
    pub async fn upload_bytes(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(body))
            .content_type(content_type)
            .send()
            .await?;

        Ok(())
    }

    /// Download a file from R2
    pub async fn download_file(&self, key: &str, dest_path: &Path) -> Result<()> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let body = response.body.collect().await?.into_bytes();
        let data = body.as_ref();

        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| Error::Io(e))?;
        }

        // Write file
        tokio::fs::write(dest_path, data).await.map_err(|e| Error::Io(e))?;

        Ok(())
    }

    /// Download bytes from R2
    pub async fn download_bytes(&self, key: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let body = response.body.collect().await?.into_bytes();

        Ok(body.to_vec())
    }

    /// List objects in the bucket
    pub async fn list_objects(&self, prefix: Option<&str>) -> Result<Vec<ObjectInfo>> {
        let response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .set_prefix(prefix.map(|s| s.to_string()))
            .send()
            .await?;

        let objects = response
            .contents()
            .iter()
            .map(|obj| ObjectInfo {
                key: obj.key().unwrap_or("").to_string(),
                size: obj.size().unwrap_or(0),
                last_modified: obj.last_modified().unwrap().to_owned(),
                etag: obj.e_tag().unwrap_or("").to_string(),
            })
            .collect();

        Ok(objects)
    }

    /// Delete an object from R2
    pub async fn delete_object(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    /// Delete multiple objects
    pub async fn delete_objects(&self, keys: Vec<String>) -> Result<()> {
        for key in keys {
            self.delete_object(&key).await?;
        }
        Ok(())
    }

    /// Check if an object exists
    pub async fn object_exists(&self, key: &str) -> Result<bool> {
        match self.client.head_object().bucket(&self.bucket).key(key).send().await {
            Ok(_) => Ok(true),
            Err(e) => {
                // Check if it's a "not found" error
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("nosuchkey") || err_str.contains("not found") || err_str.contains("404") {
                    Ok(false)
                } else {
                    Err(Error::R2Operation(e.to_string()))
                }
            }
        }
    }

    /// Get object metadata
    pub async fn head_object(&self, key: &str) -> Result<ObjectMetadata> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: response.content_length().unwrap_or(0),
            content_type: response.content_type().unwrap_or("").to_string(),
            last_modified: response.last_modified().unwrap().to_owned(),
            etag: response.e_tag().unwrap_or("").to_string(),
        })
    }

    /// Copy an object within R2
    pub async fn copy_object(&self, source_key: &str, dest_key: &str) -> Result<()> {
        self.client
            .copy_object()
            .bucket(&self.bucket)
            .key(dest_key)
            .copy_source(format!("{}/{}", self.bucket, source_key))
            .send()
            .await?;

        Ok(())
    }

    /// Get the bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

/// Object information
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub size: i64,
    pub last_modified: aws_smithy_types::DateTime,
    pub etag: String,
}

/// Object metadata
#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: i64,
    pub content_type: String,
    pub last_modified: aws_smithy_types::DateTime,
    pub etag: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_info() {
        let info = ObjectInfo {
            key: "test/file.txt".to_string(),
            size: 1024,
            last_modified: aws_smithy_types::DateTime::from(0),
            etag: "abc123".to_string(),
        };

        assert_eq!(info.key, "test/file.txt");
        assert_eq!(info.size, 1024);
    }
}
