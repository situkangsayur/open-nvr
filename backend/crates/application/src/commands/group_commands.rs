use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::*;
use crate::dto::*;
use std::sync::Arc;
use uuid::Uuid;

pub struct GroupCommandService {
    group_repo: Arc<dyn CameraGroupRepository>,
    audit_repo: Arc<dyn AuditRepository>,
}

impl GroupCommandService {
    pub fn new(
        group_repo: Arc<dyn CameraGroupRepository>,
        audit_repo: Arc<dyn AuditRepository>,
    ) -> Self {
        Self { group_repo, audit_repo }
    }

    pub async fn create_group(
        &self,
        req: CreateGroupRequest,
        user_id: &str,
    ) -> Result<CameraGroup, DomainError> {
        req.validate().map_err(DomainError::Validation)?;

        let group = CameraGroup::new(req.name, req.description);
        self.group_repo.create(&group).await?;

        let audit = AuditLog::new(
            "group.create".into(),
            "camera_group".into(),
            Some(group.id.to_string()),
        )
        .with_user(user_id.to_string(), None);
        let _ = self.audit_repo.log(&audit).await;

        Ok(group)
    }

    pub async fn delete_group(
        &self,
        id: Uuid,
        user_id: &str,
    ) -> Result<(), DomainError> {
        self.group_repo.find_by_id(id).await?
            .ok_or(DomainError::NotFound { entity_type: "camera_group".into(), id })?;

        self.group_repo.delete(id).await?;

        let audit = AuditLog::new(
            "group.delete".into(),
            "camera_group".into(),
            Some(id.to_string()),
        )
        .with_user(user_id.to_string(), None);
        let _ = self.audit_repo.log(&audit).await;

        Ok(())
    }
}
