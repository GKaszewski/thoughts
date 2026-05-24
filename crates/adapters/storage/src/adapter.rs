use async_trait::async_trait;
use domain::{
    errors::DomainError,
    ports::{DataStream, MediaStore},
};
use futures::stream::StreamExt;
use object_store::{path::Path, Error as OsError, ObjectStore};
use std::sync::Arc;

pub struct ObjectStorageAdapter {
    store: Arc<dyn ObjectStore>,
    prefix: String,
}

fn validate_key(key: &str) -> Result<(), DomainError> {
    if key.is_empty() {
        return Err(DomainError::InvalidInput(
            "storage key must not be empty".into(),
        ));
    }
    if key.starts_with('/') {
        return Err(DomainError::InvalidInput(format!(
            "storage key must not start with '/': {key}"
        )));
    }
    if key.split('/').any(|seg| seg == ".." || seg == ".") {
        return Err(DomainError::InvalidInput(format!(
            "storage key contains invalid path segment: {key}"
        )));
    }
    Ok(())
}

fn map_os_err(e: OsError) -> DomainError {
    match e {
        OsError::NotFound { .. } => DomainError::NotFound,
        e => DomainError::Internal(e.to_string()),
    }
}

impl ObjectStorageAdapter {
    pub fn new(
        store: Arc<dyn ObjectStore>,
        prefix: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let prefix = prefix.into();
        if !prefix.is_empty() {
            validate_key(&prefix)?;
        }
        Ok(Self { store, prefix })
    }

    fn path(&self, key: &str) -> Path {
        if self.prefix.is_empty() {
            Path::from(key)
        } else {
            Path::from(format!("{}/{key}", self.prefix))
        }
    }
}

#[async_trait]
impl MediaStore for ObjectStorageAdapter {
    async fn put(&self, key: &str, data: DataStream) -> Result<(), DomainError> {
        validate_key(key)?;
        let path = self.path(key);
        let mut upload = self
            .store
            .put_multipart(&path)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;
        let mut stream = data;
        while let Some(result) = stream.next().await {
            match result {
                Ok(bytes) => {
                    if let Err(e) = upload.put_part(bytes.into()).await {
                        let _ = upload.abort().await;
                        return Err(DomainError::Internal(e.to_string()));
                    }
                }
                Err(e) => {
                    let _ = upload.abort().await;
                    return Err(e);
                }
            }
        }
        upload
            .complete()
            .await
            .map(|_| ())
            .map_err(|e| DomainError::Internal(e.to_string()))
    }

    async fn get(&self, key: &str) -> Result<DataStream, DomainError> {
        validate_key(key)?;
        let path = self.path(key);
        let result = self.store.get(&path).await.map_err(map_os_err)?;
        let s = result
            .into_stream()
            .map(|r| r.map_err(|e| DomainError::Internal(e.to_string())));
        Ok(Box::pin(s))
    }

    async fn delete(&self, key: &str) -> Result<(), DomainError> {
        validate_key(key)?;
        let path = self.path(key);
        match self.store.delete(&path).await {
            Ok(()) => Ok(()),
            Err(OsError::NotFound { .. }) => Ok(()),
            Err(e) => Err(DomainError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures::stream;
    use object_store::memory::InMemory;

    fn make_adapter() -> ObjectStorageAdapter {
        ObjectStorageAdapter::new(Arc::new(InMemory::new()), "test").unwrap()
    }

    fn one_shot(data: &'static [u8]) -> DataStream {
        Box::pin(stream::once(async move { Ok(Bytes::from(data)) }))
    }

    #[tokio::test]
    async fn put_get_roundtrip() {
        let a = make_adapter();
        a.put("hello.txt", one_shot(b"world")).await.unwrap();
        let mut s = a.get("hello.txt").await.unwrap();
        let mut out = Vec::new();
        while let Some(chunk) = s.next().await {
            out.extend_from_slice(&chunk.unwrap());
        }
        assert_eq!(out, b"world");
    }

    #[tokio::test]
    async fn get_missing_is_not_found() {
        let a = make_adapter();
        assert!(matches!(
            a.get("nope.txt").await,
            Err(DomainError::NotFound)
        ));
    }

    #[tokio::test]
    async fn delete_is_idempotent() {
        let a = make_adapter();
        a.delete("nope.txt").await.unwrap();
    }

    #[tokio::test]
    async fn delete_removes_key() {
        let a = make_adapter();
        a.put("file.txt", one_shot(b"data")).await.unwrap();
        a.delete("file.txt").await.unwrap();
        assert!(matches!(
            a.get("file.txt").await,
            Err(DomainError::NotFound)
        ));
    }

    #[tokio::test]
    async fn put_overwrites_existing() {
        let a = make_adapter();
        a.put("file.txt", one_shot(b"v1")).await.unwrap();
        a.put("file.txt", one_shot(b"v2")).await.unwrap();
        let mut s = a.get("file.txt").await.unwrap();
        let mut out = Vec::new();
        while let Some(chunk) = s.next().await {
            out.extend_from_slice(&chunk.unwrap());
        }
        assert_eq!(out, b"v2");
    }

    #[tokio::test]
    async fn rejects_empty_key() {
        let a = make_adapter();
        assert!(matches!(
            a.put("", one_shot(b"x")).await,
            Err(DomainError::InvalidInput(_))
        ));
        assert!(matches!(a.get("").await, Err(DomainError::InvalidInput(_))));
        assert!(matches!(
            a.delete("").await,
            Err(DomainError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn rejects_absolute_key() {
        let a = make_adapter();
        assert!(matches!(
            a.put("/etc/passwd", one_shot(b"x")).await,
            Err(DomainError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn rejects_path_traversal() {
        let a = make_adapter();
        assert!(matches!(
            a.get("../escape").await,
            Err(DomainError::InvalidInput(_))
        ));
        assert!(matches!(
            a.get("a/../../../etc").await,
            Err(DomainError::InvalidInput(_))
        ));
    }

    #[test]
    fn new_rejects_traversal_prefix() {
        assert!(matches!(
            ObjectStorageAdapter::new(Arc::new(InMemory::new()), "../evil"),
            Err(DomainError::InvalidInput(_))
        ));
    }

    #[test]
    fn new_rejects_absolute_prefix() {
        assert!(matches!(
            ObjectStorageAdapter::new(Arc::new(InMemory::new()), "/root"),
            Err(DomainError::InvalidInput(_))
        ));
    }

    #[test]
    fn new_accepts_empty_prefix() {
        assert!(ObjectStorageAdapter::new(Arc::new(InMemory::new()), "").is_ok());
    }
}
