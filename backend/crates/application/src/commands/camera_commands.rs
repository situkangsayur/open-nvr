use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::*;
use crate::dto::*;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub struct CameraCommandService {
    camera_repo: Arc<dyn CameraRepository>,
    audit_repo: Arc<dyn AuditRepository>,
}

impl CameraCommandService {
    pub fn new(
        camera_repo: Arc<dyn CameraRepository>,
        audit_repo: Arc<dyn AuditRepository>,
    ) -> Self {
        Self { camera_repo, audit_repo }
    }

    pub async fn create_camera(
        &self,
        req: CreateCameraRequest,
        user_id: &str,
        user_email: Option<&str>,
    ) -> Result<Camera, DomainError> {
        let protocol = req.protocol_type.parse::<ProtocolType>()
            .map_err(|e| DomainError::Validation(e))?;

        let mut camera = Camera::new(req.name.clone(), protocol, req.stream_url);
        camera.brand = req.brand;
        camera.model = req.model;
        camera.sub_stream_url = req.sub_stream_url;
        camera.onvif_url = req.onvif_url;
        camera.ptz_capable = req.ptz_capable.unwrap_or(false);
        camera.audio_capable = req.audio_capable.unwrap_or(false);
        camera.group_id = req.group_id;

        if let Some(ct) = req.connection_type {
            camera.connection_type = ct.parse::<ConnectionType>()
                .map_err(|e| DomainError::Validation(e))?;
        }
        if let Some(rm) = req.recording_mode {
            camera.recording_mode = rm.parse::<RecordingMode>()
                .map_err(|e| DomainError::Validation(e))?;
        }

        // TODO: encrypt credentials if username/password provided

        self.camera_repo.create(&camera).await?;

        // Audit log
        let audit = AuditLog::new(
            "camera.create".into(),
            "camera".into(),
            Some(camera.id.to_string()),
        )
        .with_user(user_id.to_string(), user_email.map(String::from))
        .with_details(serde_json::json!({
            "camera_name": camera.name,
            "protocol": camera.protocol_type.to_string(),
        }));
        let _ = self.audit_repo.log(&audit).await;

        info!(camera_id = %camera.id, name = %camera.name, "Camera created");
        Ok(camera)
    }

    pub async fn update_camera(
        &self,
        id: Uuid,
        req: UpdateCameraRequest,
        user_id: &str,
        user_email: Option<&str>,
    ) -> Result<Camera, DomainError> {
        let mut camera = self.camera_repo.find_by_id(id).await?
            .ok_or(DomainError::NotFound { entity_type: "camera".into(), id })?;

        if let Some(name) = req.name { camera.name = name; }
        if let Some(brand) = req.brand { camera.brand = Some(brand); }
        if let Some(model) = req.model { camera.model = Some(model); }
        if let Some(url) = req.stream_url { camera.stream_url = url; }
        if let Some(url) = req.sub_stream_url { camera.sub_stream_url = Some(url); }
        if let Some(url) = req.onvif_url { camera.onvif_url = Some(url); }
        if let Some(ptz) = req.ptz_capable { camera.ptz_capable = ptz; }
        if let Some(audio) = req.audio_capable { camera.audio_capable = audio; }
        if let Some(gid) = req.group_id { camera.group_id = Some(gid); }
        if let Some(ct) = req.connection_type {
            camera.connection_type = ct.parse::<ConnectionType>()
                .map_err(|e| DomainError::Validation(e))?;
        }
        if let Some(rm) = req.recording_mode {
            camera.recording_mode = rm.parse::<RecordingMode>()
                .map_err(|e| DomainError::Validation(e))?;
        }

        camera.updated_at = chrono::Utc::now();
        self.camera_repo.update(&camera).await?;

        let audit = AuditLog::new(
            "camera.update".into(),
            "camera".into(),
            Some(camera.id.to_string()),
        )
        .with_user(user_id.to_string(), user_email.map(String::from));
        let _ = self.audit_repo.log(&audit).await;

        info!(camera_id = %camera.id, "Camera updated");
        Ok(camera)
    }

    pub async fn delete_camera(
        &self,
        id: Uuid,
        user_id: &str,
        user_email: Option<&str>,
    ) -> Result<(), DomainError> {
        let camera = self.camera_repo.find_by_id(id).await?
            .ok_or(DomainError::NotFound { entity_type: "camera".into(), id })?;

        self.camera_repo.delete(id).await?;

        let audit = AuditLog::new(
            "camera.delete".into(),
            "camera".into(),
            Some(id.to_string()),
        )
        .with_user(user_id.to_string(), user_email.map(String::from))
        .with_details(serde_json::json!({ "camera_name": camera.name }));
        let _ = self.audit_repo.log(&audit).await;

        info!(camera_id = %id, "Camera deleted");
        Ok(())
    }
}
