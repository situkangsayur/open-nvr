use async_trait::async_trait;
use crate::entities::{BoundingBox, DetectionEventType};
use crate::errors::DomainError;

#[derive(Debug, Clone)]
pub struct Detection {
    pub event_type: DetectionEventType,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
    pub label: String,
}

#[async_trait]
pub trait DetectionEngine: Send + Sync {
    async fn detect(&self, frame: &[u8], width: u32, height: u32) -> Result<Vec<Detection>, DomainError>;
    fn supported_types(&self) -> Vec<DetectionEventType>;
}
