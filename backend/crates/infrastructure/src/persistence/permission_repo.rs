use async_trait::async_trait;
use open_nvr_domain::entities::UserCameraAccess;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::UserPermissionRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgUserPermissionRepository {
    pool: PgPool,
}

impl PgUserPermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserPermissionRepository for PgUserPermissionRepository {
    async fn find_by_user(&self, user_id: &str) -> Result<Vec<UserCameraAccess>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, user_id, camera_id, can_view, can_ptz, can_playback, can_export, granted_by, created_at FROM user_camera_access WHERE user_id = $1 ORDER BY created_at"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(access_from_row).collect())
    }

    async fn find_by_camera(&self, camera_id: Uuid) -> Result<Vec<UserCameraAccess>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, user_id, camera_id, can_view, can_ptz, can_playback, can_export, granted_by, created_at FROM user_camera_access WHERE camera_id = $1 ORDER BY user_id"
        )
        .bind(camera_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(access_from_row).collect())
    }

    async fn grant(&self, access: &UserCameraAccess) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO user_camera_access (id, user_id, camera_id, can_view, can_ptz, can_playback, can_export, granted_by, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) ON CONFLICT (user_id, camera_id) DO UPDATE SET can_view = $4, can_ptz = $5, can_playback = $6, can_export = $7, granted_by = $8"
        )
        .bind(access.id)
        .bind(&access.user_id)
        .bind(access.camera_id)
        .bind(access.can_view)
        .bind(access.can_ptz)
        .bind(access.can_playback)
        .bind(access.can_export)
        .bind(&access.granted_by)
        .bind(access.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn revoke(&self, user_id: &str, camera_id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM user_camera_access WHERE user_id = $1 AND camera_id = $2")
            .bind(user_id)
            .bind(camera_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: &str) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM user_camera_access WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

fn access_from_row(row: &sqlx::postgres::PgRow) -> UserCameraAccess {
    UserCameraAccess {
        id: row.get("id"),
        user_id: row.get("user_id"),
        camera_id: row.get("camera_id"),
        can_view: row.get("can_view"),
        can_ptz: row.get("can_ptz"),
        can_playback: row.get("can_playback"),
        can_export: row.get("can_export"),
        granted_by: row.get("granted_by"),
        created_at: row.get("created_at"),
    }
}
