use anyhow::{Context, Result};
use object_store::local::LocalFileSystem;
use object_store::ObjectStore;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub backend: String,
    pub local_path: Option<String>,
    pub s3_endpoint: Option<String>,
    pub s3_access_key_id: Option<String>,
    pub s3_secret_access_key: Option<String>,
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
}

pub fn build_store(config: &StorageConfig) -> Result<Arc<dyn ObjectStore>> {
    match config.backend.as_str() {
        "local" => {
            let path = config
                .local_path
                .as_deref()
                .context("STORAGE_PATH must be set when STORAGE_BACKEND=local")?;
            std::fs::create_dir_all(path)
                .with_context(|| format!("failed to create storage dir: {path}"))?;
            let store = LocalFileSystem::new_with_prefix(path)?;
            Ok(Arc::new(store))
        }
        #[cfg(feature = "s3")]
        "s3" => {
            use object_store::aws::AmazonS3Builder;
            let store = AmazonS3Builder::new()
                .with_endpoint(
                    config
                        .s3_endpoint
                        .as_deref()
                        .context("S3_ENDPOINT must be set")?,
                )
                .with_access_key_id(
                    config
                        .s3_access_key_id
                        .as_deref()
                        .context("S3_ACCESS_KEY_ID must be set")?,
                )
                .with_secret_access_key(
                    config
                        .s3_secret_access_key
                        .as_deref()
                        .context("S3_SECRET_ACCESS_KEY must be set")?,
                )
                .with_bucket_name(
                    config
                        .s3_bucket
                        .as_deref()
                        .context("S3_BUCKET must be set")?,
                )
                .with_region(config.s3_region.as_deref().unwrap_or("us-east-1"))
                .with_allow_http(true)
                .build()?;
            Ok(Arc::new(store))
        }
        other => anyhow::bail!(
            "unknown STORAGE_BACKEND={other:?}; supported: local{}",
            if cfg!(feature = "s3") { ", s3" } else { "" },
        ),
    }
}
