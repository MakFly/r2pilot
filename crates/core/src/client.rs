//! R2 Client implementation using AWS S3 SDK

use crate::error::{Error, Result};
use aws_sdk_s3::{
    config::{Credentials, Region},
    primitives::ByteStream,
    Client,
};
use std::time::Duration;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

// === Multipart Upload Types ===

/// Configuration for multipart upload
#[derive(Debug, Clone)]
pub struct MultipartUploadConfig {
    /// Chunk size in bytes (default: 100MB)
    pub chunk_size: usize,
    /// Maximum concurrent uploads (default: 5)
    pub concurrent_parts: usize,
}

impl Default for MultipartUploadConfig {
    fn default() -> Self {
        Self {
            chunk_size: 100 * 1024 * 1024, // 100MB
            concurrent_parts: 5,
        }
    }
}

/// Progress information for multipart upload
#[derive(Debug, Clone)]
pub struct MultipartUploadProgress {
    pub upload_id: String,
    pub total_bytes: u64,
    pub uploaded_bytes: u64,
    pub completed_parts: usize,
    pub total_parts: usize,
}

/// A completed part in a multipart upload
#[derive(Debug, Clone)]
pub struct CompletedPart {
    pub part_number: i32,
    pub etag: String,
}

/// R2 client for managing Cloudflare R2 storage
pub struct R2Client {
    client: Client,
    bucket: String,
    #[allow(dead_code)]
    endpoint: String,
    #[allow(dead_code)]
    access_key_id: String,
    #[allow(dead_code)]
    secret_access_key: String,
}

impl R2Client {
    /// Create a new R2 client
    pub async fn new(
        endpoint: String,
        access_key_id: String,
        secret_access_key: String,
        bucket: String,
    ) -> Result<Self> {
        let endpoint_clone = endpoint.clone();
        // Create credentials
        let credentials = Credentials::new(&access_key_id, &secret_access_key, None, None, "r2pilot");

        // Build AWS config for R2 (S3-compatible)
        let config_builder = aws_sdk_s3::Config::builder()
            .endpoint_url(&endpoint)
            .region(Region::new("auto".to_string()))
            .credentials_provider(credentials);

        let config = config_builder.build();

        let client = Client::from_conf(config);

        Ok(Self {
            client,
            bucket,
            endpoint: endpoint_clone,
            access_key_id,
            secret_access_key,
        })
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

    // === Multipart Upload Operations ===

    /// Initiate a multipart upload
    pub async fn create_multipart_upload(&self, key: &str, content_type: &str) -> Result<String> {
        let response = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .send()
            .await?;

        response
            .upload_id()
            .map(|id| id.to_string())
            .ok_or_else(|| Error::MultipartUpload("No upload ID returned".to_string()))
    }

    /// Upload a single part in a multipart upload
    pub async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: i32,
        body: Vec<u8>,
    ) -> Result<CompletedPart> {
        let response = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .part_number(part_number)
            .body(ByteStream::from(body))
            .send()
            .await?;

        let etag = response
            .e_tag()
            .map(|etag| etag.to_string())
            .ok_or_else(|| Error::MultipartUpload("No ETag returned for part".to_string()))?;

        Ok(CompletedPart {
            part_number,
            etag,
        })
    }

    /// Complete a multipart upload
    pub async fn complete_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<CompletedPart>,
    ) -> Result<()> {
        // Convert our CompletedPart to AWS SDK CompletedPart
        let aws_parts: Vec<aws_sdk_s3::types::CompletedPart> = parts
            .iter()
            .map(|p| {
                aws_sdk_s3::types::CompletedPart::builder()
                    .part_number(p.part_number)
                    .e_tag(&p.etag)
                    .build()
            })
            .collect();

        // Build the multipart upload with all parts
        let multipart_upload = aws_sdk_s3::types::CompletedMultipartUpload::builder()
            .set_parts(Some(aws_parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(multipart_upload)
            .send()
            .await?;

        Ok(())
    }

    /// Abort a multipart upload (cleanup on error)
    pub async fn abort_multipart_upload(&self, key: &str, upload_id: &str) -> Result<()> {
        self.client
            .abort_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .send()
            .await?;

        Ok(())
    }

    /// Upload a file using multipart upload
    pub async fn upload_file_multipart(
        &self,
        key: &str,
        file_path: &Path,
        content_type: &str,
        config: MultipartUploadConfig,
    ) -> Result<()> {
        use tokio::io::AsyncReadExt;

        // Open file and get size
        let file = File::open(file_path).await.map_err(|e| Error::Io(e))?;
        let metadata = file.metadata().await.map_err(|e| Error::Io(e))?;
        let file_size = metadata.len();

        // Calculate number of parts
        let _total_parts = (file_size as usize + config.chunk_size - 1) / config.chunk_size;

        // Initiate multipart upload
        let upload_id = self.create_multipart_upload(key, content_type).await?;

        // Read and upload parts
        let mut parts = Vec::new();
        let mut current_part = 0;

        // Reopen file for reading chunks
        let mut file = File::open(file_path).await.map_err(|e| Error::Io(e))?;

        loop {
            current_part += 1;

            // Read chunk
            let mut buffer = vec![0u8; config.chunk_size.min(100 * 1024 * 1024)];
            let n = file.read(&mut buffer).await.map_err(|e| Error::Io(e))?;

            if n == 0 {
                break;
            }

            buffer.truncate(n);

            // Upload part
            match self.upload_part(key, &upload_id, current_part, buffer).await {
                Ok(part) => parts.push(part),
                Err(e) => {
                    // Abort on error
                    let _ = self.abort_multipart_upload(key, &upload_id).await;
                    return Err(e);
                }
            }
        }

        // Complete multipart upload
        self.complete_multipart_upload(key, &upload_id, parts).await?;

        Ok(())
    }

    /// Generate a presigned URL for an object
    ///
    /// Note: For Cloudflare R2, presigned URLs require additional setup.
    /// This method returns a direct URL. For presigned URLs, ensure your bucket
    /// has the proper CORS configuration and consider using R2's built-in
    /// URL signing features.
    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String> {
        use http::Uri;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Parse endpoint to extract host
        let endpoint_uri: Uri = self.endpoint.parse()
            .map_err(|e| Error::InvalidConfig(format!("Invalid endpoint URL: {}", e)))?;

        let host = endpoint_uri.host()
            .ok_or_else(|| Error::InvalidConfig("Endpoint has no host".to_string()))?;

        // Build the object URL
        let url = format!("https://{}/{}/{}", host, self.bucket, key);

        // For R2, we'll return a simple URL structure
        // In production, you'd use AWS SDK's presigning features or implement
        // AWS Signature Version 4 manually
        let expires_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::R2Operation(format!("Time error: {}", e)))?
            .as_secs()
            + expires_in.as_secs();

        // Build presigned URL parameters (simplified - needs full AWS SigV4 implementation)
        // For now, return the direct URL with expiration info
        Ok(format!(
            "{}?expires={}",
            url,
            expires_timestamp
        ))
    }

    /// Get the bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

/// Check if a file requires multipart upload (>100MB)
pub fn requires_multipart_upload(file_size: u64) -> bool {
    file_size > 100 * 1024 * 1024
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
    use std::time::SystemTime;

    #[test]
    fn test_object_info() {
        let info = ObjectInfo {
            key: "test/file.txt".to_string(),
            size: 1024,
            last_modified: aws_smithy_types::DateTime::from(SystemTime::UNIX_EPOCH),
            etag: "abc123".to_string(),
        };

        assert_eq!(info.key, "test/file.txt");
        assert_eq!(info.size, 1024);
    }
}
