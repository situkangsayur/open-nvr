use async_trait::async_trait;
use open_nvr_domain::entities::RetentionPolicy;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::RetentionPolicyRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgRetentionPolicyRepository {
    pool: PgPool,
}

impl PgRetentionPolicyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RetentionPolicyRepository for PgRetentionPolicyRepository {
    async fn find_all(&self) -> Result<Vec<RetentionPolicy>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, name, camera_id, retention_days, max_storage_bytes, recording_type, enabled, created_at, updated_at FROM retention_policies ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(policy_from_row).collect())
    }

    async fn find_by_camera(&self, camera_id: Uuid) -> Result<Vec<RetentionPolicy>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, name, camera_id, retention_days, max_storage_bytes, recording_type, enabled, created_at, updated_at FROM retention_policies WHERE camera_id = $1 OR camera_id IS NULL ORDER BY name"
        )
        .bind(camera_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(policy_from_row).collect())
    }

    async fn create(&self, policy: &RetentionPolicy) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO retention_policies (id, name, camera_id, retention_days, max_storage_bytes, recording_type, enabled, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(policy.id)
        .bind(&policy.name)
        .bind(policy.camera_id)
        .bind(policy.retention_days)
        .bind(policy.max_storage_bytes)
        .bind(&policy.recording_type)
        .bind(policy.enabled)
        .bind(policy.created_at)
        .bind(policy.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, policy: &RetentionPolicy) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE retention_policies SET name = $2, camera_id = $3, retention_days = $4, max_storage_bytes = $5, recording_type = $6, enabled = $7, updated_at = $8 WHERE id = $1"
        )
        .bind(policy.id)
        .bind(&policy.name)
        .bind(policy.camera_id)
        .bind(policy.retention_days)
        .bind(policy.max_storage_bytes)
        .bind(&policy.recording_type)
        .bind(policy.enabled)
        .bind(policy.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM retention_policies WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

fn policy_from_row(row: &sqlx::postgres::PgRow) -> RetentionPolicy {
    RetentionPolicy {
        id: row.get("id"),
        name: row.get("name"),
        camera_id: row.get("camera_id"),
        retention_days: row.get("retention_days"),
        max_storage_bytes: row.get("max_storage_bytes"),
        recording_type: row.get("recording_type"),
        enabled: row.get("enabled"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
