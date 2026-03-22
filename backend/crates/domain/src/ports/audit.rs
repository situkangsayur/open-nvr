use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::entities::{AuditLog, NetworkEvent};
use crate::errors::DomainError;

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn log(&self, entry: &AuditLog) -> Result<(), DomainError>;
    async fn find_by_user(&self, user_id: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<AuditLog>, DomainError>;
    async fn find_by_resource(&self, resource_type: &str, resource_id: &str) -> Result<Vec<AuditLog>, DomainError>;
    async fn find_recent(&self, limit: i64) -> Result<Vec<AuditLog>, DomainError>;
}

#[async_trait]
pub trait NetworkEventRepository: Send + Sync {
    async fn create(&self, event: &NetworkEvent) -> Result<(), DomainError>;
    async fn find_unresolved(&self) -> Result<Vec<NetworkEvent>, DomainError>;
    async fn find_by_severity(&self, severity: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<NetworkEvent>, DomainError>;
    async fn resolve(&self, id: uuid::Uuid, resolved_by: &str) -> Result<(), DomainError>;
}
