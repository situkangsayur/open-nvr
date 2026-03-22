use async_trait::async_trait;
use crate::errors::DomainError;

#[async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put_object(&self, key: &str, data: &[u8], content_type: &str) -> Result<(), DomainError>;
    async fn get_object(&self, key: &str) -> Result<Vec<u8>, DomainError>;
    async fn delete_object(&self, key: &str) -> Result<(), DomainError>;
    async fn presigned_url(&self, key: &str, expiry_secs: u64) -> Result<String, DomainError>;
    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>, DomainError>;
    async fn get_total_size(&self, prefix: &str) -> Result<u64, DomainError>;
}
