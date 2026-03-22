use async_trait::async_trait;
use crate::errors::DomainError;

#[async_trait]
pub trait CachePort: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, DomainError>;
    async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), DomainError>;
    async fn delete(&self, key: &str) -> Result<(), DomainError>;
    async fn exists(&self, key: &str) -> Result<bool, DomainError>;
    async fn publish(&self, channel: &str, message: &str) -> Result<(), DomainError>;
}
