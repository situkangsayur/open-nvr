use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::entities::*;
use crate::errors::DomainError;

#[async_trait]
pub trait CameraRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Camera>, DomainError>;
    async fn find_all(&self) -> Result<Vec<Camera>, DomainError>;
    async fn find_by_group(&self, group_id: Uuid) -> Result<Vec<Camera>, DomainError>;
    async fn find_by_status(&self, status: CameraStatus) -> Result<Vec<Camera>, DomainError>;
    async fn create(&self, camera: &Camera) -> Result<(), DomainError>;
    async fn update(&self, camera: &Camera) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
    async fn update_status(&self, id: Uuid, status: CameraStatus) -> Result<(), DomainError>;
}

#[async_trait]
pub trait CameraGroupRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CameraGroup>, DomainError>;
    async fn find_all(&self) -> Result<Vec<CameraGroup>, DomainError>;
    async fn create(&self, group: &CameraGroup) -> Result<(), DomainError>;
    async fn update(&self, group: &CameraGroup) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait RecordingRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Recording>, DomainError>;
    async fn find_by_camera(&self, camera_id: Uuid, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Recording>, DomainError>;
    async fn create(&self, recording: &Recording) -> Result<(), DomainError>;
    async fn update(&self, recording: &Recording) -> Result<(), DomainError>;
    async fn find_active_by_camera(&self, camera_id: Uuid) -> Result<Option<Recording>, DomainError>;
    async fn find_before_date(&self, before: DateTime<Utc>) -> Result<Vec<Recording>, DomainError>;
}

#[async_trait]
pub trait RecordingSegmentRepository: Send + Sync {
    async fn find_by_recording(&self, recording_id: Uuid) -> Result<Vec<RecordingSegment>, DomainError>;
    async fn create(&self, segment: &RecordingSegment) -> Result<(), DomainError>;
    async fn update(&self, segment: &RecordingSegment) -> Result<(), DomainError>;
    async fn delete_by_recording(&self, recording_id: Uuid) -> Result<Vec<String>, DomainError>;
}

#[async_trait]
pub trait DetectionEventRepository: Send + Sync {
    async fn find_by_camera(&self, camera_id: Uuid, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<DetectionEvent>, DomainError>;
    async fn find_by_type(&self, event_type: DetectionEventType, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<DetectionEvent>, DomainError>;
    async fn create(&self, event: &DetectionEvent) -> Result<(), DomainError>;
}

#[async_trait]
pub trait DetectionZoneRepository: Send + Sync {
    async fn find_by_camera(&self, camera_id: Uuid) -> Result<Vec<DetectionZone>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<DetectionZone>, DomainError>;
    async fn create(&self, zone: &DetectionZone) -> Result<(), DomainError>;
    async fn update(&self, zone: &DetectionZone) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait GridLayoutRepository: Send + Sync {
    async fn find_by_user(&self, user_id: &str) -> Result<Vec<GridLayout>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<GridLayout>, DomainError>;
    async fn create(&self, layout: &GridLayout) -> Result<(), DomainError>;
    async fn update(&self, layout: &GridLayout) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait RetentionPolicyRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<RetentionPolicy>, DomainError>;
    async fn find_by_camera(&self, camera_id: Uuid) -> Result<Vec<RetentionPolicy>, DomainError>;
    async fn create(&self, policy: &RetentionPolicy) -> Result<(), DomainError>;
    async fn update(&self, policy: &RetentionPolicy) -> Result<(), DomainError>;
    async fn delete(&self, id: Uuid) -> Result<(), DomainError>;
}
