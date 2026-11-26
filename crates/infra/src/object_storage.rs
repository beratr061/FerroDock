use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::{AppError, AppResult};
use futures_util::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt, PutPayload, WriteMultipart};
use zeroize::Zeroizing;

const PROBE_KEY: &str = ".ferrodock-probe";
const PART_SIZE: usize = 8 * 1024 * 1024;
const PARTS_IN_FLIGHT: usize = 2;

#[derive(Debug, Clone)]
pub struct S3Target {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: Zeroizing<String>,
    pub path_style: bool,
}

#[derive(Debug, Clone)]
pub enum StorageTarget {
    Local(PathBuf),
    S3(S3Target),
}

#[derive(Debug, Clone)]
pub struct ObjectStorage {
    store: Arc<dyn ObjectStore>,
    prefix: String,
}

fn storage_error(error: object_store::Error) -> AppError {
    let code = match error {
        object_store::Error::NotFound { .. } => "BACKUP_OBJECT_MISSING",
        _ => "BACKUP_STORAGE_FAILED",
    };
    AppError::validation(code).with_source(error)
}

fn s3(target: S3Target) -> AppResult<Arc<dyn ObjectStore>> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut builder = AmazonS3Builder::new()
        .with_region(target.region)
        .with_bucket_name(target.bucket)
        .with_access_key_id(target.access_key_id)
        .with_secret_access_key(target.secret_access_key.as_str())
        .with_virtual_hosted_style_request(!target.path_style);
    if let Some(endpoint) = target.endpoint {
        builder = builder
            .with_allow_http(endpoint.starts_with("http://"))
            .with_endpoint(endpoint);
    }
    Ok(Arc::new(builder.build().map_err(storage_error)?))
}

async fn local(dir: PathBuf) -> AppResult<Arc<dyn ObjectStore>> {
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(AppError::internal)?;
    let store = LocalFileSystem::new_with_prefix(dir).map_err(storage_error)?;
    Ok(Arc::new(store.with_automatic_cleanup(true)))
}

impl ObjectStorage {
    pub async fn open(target: StorageTarget, prefix: &str) -> AppResult<Self> {
        let store = match target {
            StorageTarget::Local(dir) => local(dir).await?,
            StorageTarget::S3(target) => s3(target)?,
        };
        Ok(Self {
            store,
            prefix: prefix.trim_matches('/').to_owned(),
        })
    }

    fn location(&self, key: &str) -> AppResult<Path> {
        let full = if self.prefix.is_empty() {
            key.to_owned()
        } else {
            format!("{}/{key}", self.prefix)
        };
        Path::parse(full).map_err(|_| AppError::validation("BACKUP_KEY_INVALID"))
    }

    pub async fn put(&self, key: &str, bytes: Vec<u8>) -> AppResult<()> {
        let location = self.location(key)?;
        self.store
            .put(&location, PutPayload::from(bytes))
            .await
            .map(|_| ())
            .map_err(storage_error)
    }

    pub async fn writer(&self, key: &str) -> AppResult<ObjectWriter> {
        let location = self.location(key)?;
        let upload = self
            .store
            .put_multipart(&location)
            .await
            .map_err(storage_error)?;
        Ok(ObjectWriter {
            upload: WriteMultipart::new_with_chunk_size(upload, PART_SIZE),
            written: 0,
        })
    }

    pub async fn reader(&self, key: &str) -> AppResult<BoxStream<'static, AppResult<Vec<u8>>>> {
        let location = self.location(key)?;
        let result = self.store.get(&location).await.map_err(storage_error)?;
        Ok(result
            .into_stream()
            .map(|chunk| chunk.map(|bytes| bytes.to_vec()).map_err(storage_error))
            .boxed())
    }

    pub async fn get(&self, key: &str) -> AppResult<Vec<u8>> {
        let location = self.location(key)?;
        let result = self.store.get(&location).await.map_err(storage_error)?;
        let bytes = result.bytes().await.map_err(storage_error)?;
        Ok(bytes.to_vec())
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let location = self.location(key)?;
        match self.store.delete(&location).await {
            Err(object_store::Error::NotFound { .. }) | Ok(()) => Ok(()),
            Err(error) => Err(storage_error(error)),
        }
    }

    pub async fn probe(&self) -> AppResult<()> {
        self.put(PROBE_KEY, b"ok".to_vec()).await?;
        self.get(PROBE_KEY).await?;
        self.delete(PROBE_KEY).await
    }

    pub async fn keys(&self) -> AppResult<Vec<String>> {
        let prefix = (!self.prefix.is_empty()).then(|| Path::from(self.prefix.as_str()));
        let objects: Vec<_> = self
            .store
            .list(prefix.as_ref())
            .try_collect()
            .await
            .map_err(storage_error)?;
        Ok(objects
            .into_iter()
            .map(|meta| meta.location.to_string())
            .collect())
    }
}

#[derive(Debug)]
pub struct ObjectWriter {
    upload: WriteMultipart,
    written: u64,
}

impl ObjectWriter {
    pub async fn write(&mut self, bytes: &[u8]) -> AppResult<()> {
        self.upload
            .wait_for_capacity(PARTS_IN_FLIGHT)
            .await
            .map_err(storage_error)?;
        self.upload.write(bytes);
        self.written += bytes.len() as u64;
        Ok(())
    }

    pub async fn finish(self) -> AppResult<u64> {
        self.upload.finish().await.map_err(storage_error)?;
        Ok(self.written)
    }

    pub async fn abort(self) {
        if let Err(error) = self.upload.abort().await {
            tracing::warn!(%error, "multipart upload not aborted");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stores_reads_and_deletes_locally() -> AppResult<()> {
        let dir = std::env::temp_dir().join(format!("fd-objects-{}", ferrodock_core::Id::new()));
        let storage = ObjectStorage::open(StorageTarget::Local(dir.clone()), "org/db").await?;
        storage.probe().await?;
        storage.put("one.dump", b"data".to_vec()).await?;
        assert_eq!(storage.get("one.dump").await?, b"data");
        assert_eq!(storage.keys().await?, vec!["org/db/one.dump".to_owned()]);
        storage.delete("one.dump").await?;
        storage.delete("one.dump").await?;
        assert_eq!(
            storage.get("one.dump").await.err().map(|e| e.code()),
            Some("BACKUP_OBJECT_MISSING")
        );
        assert!(storage.location("../escape").is_err());
        let _ = tokio::fs::remove_dir_all(dir).await;
        Ok(())
    }
}
