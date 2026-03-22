use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound { entity_type: String, id: Uuid },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate entity: {0}")]
    Duplicate(String),

    #[error("Camera connection error: {0}")]
    CameraConnection(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
