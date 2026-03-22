use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct CameraQueryService {
    camera_repo: Arc<dyn CameraRepository>,
}

impl CameraQueryService {
    pub fn new(camera_repo: Arc<dyn CameraRepository>) -> Self {
        Self { camera_repo }
    }

    pub async fn get_camera(&self, id: Uuid) -> Result<Camera, DomainError> {
        self.camera_repo.find_by_id(id).await?
            .ok_or(DomainError::NotFound { entity_type: "camera".into(), id })
    }

    pub async fn list_cameras(&self) -> Result<Vec<Camera>, DomainError> {
        self.camera_repo.find_all().await
    }

    pub async fn list_by_group(&self, group_id: Uuid) -> Result<Vec<Camera>, DomainError> {
        self.camera_repo.find_by_group(group_id).await
    }

    pub async fn list_by_status(&self, status: CameraStatus) -> Result<Vec<Camera>, DomainError> {
        self.camera_repo.find_by_status(status).await
    }
}
