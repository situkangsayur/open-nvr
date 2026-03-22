use async_trait::async_trait;
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::CameraGroupRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgCameraGroupRepository {
    pool: PgPool,
}

impl PgCameraGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn group_from_row(r: sqlx::postgres::PgRow) -> CameraGroup {
    CameraGroup {
        id: r.get("id"),
        name: r.get("name"),
        description: r.get("description"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }
}

#[async_trait]
impl CameraGroupRepository for PgCameraGroupRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<CameraGroup>, DomainError> {
        let row = sqlx::query(
            "SELECT id, name, description, created_at, updated_at FROM camera_groups WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(group_from_row))
    }

    async fn find_all(&self) -> Result<Vec<CameraGroup>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, name, description, created_at, updated_at FROM camera_groups ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(group_from_row).collect())
    }

    async fn create(&self, group: &CameraGroup) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO camera_groups (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(group.id)
        .bind(&group.name)
        .bind(&group.description)
        .bind(group.created_at)
        .bind(group.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, group: &CameraGroup) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE camera_groups SET name = $2, description = $3, updated_at = $4 WHERE id = $1",
        )
        .bind(group.id)
        .bind(&group.name)
        .bind(&group.description)
        .bind(group.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM camera_groups WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
