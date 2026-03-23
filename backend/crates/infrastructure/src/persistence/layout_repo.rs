use async_trait::async_trait;
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::GridLayoutRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgGridLayoutRepository {
    pool: PgPool,
}

impl PgGridLayoutRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GridLayoutRepository for PgGridLayoutRepository {
    async fn find_by_user(&self, user_id: &str) -> Result<Vec<GridLayout>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, user_id, name, layout_type, camera_positions, is_default, created_at, updated_at FROM grid_layouts WHERE user_id = $1 ORDER BY name"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(layout_from_row).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<GridLayout>, DomainError> {
        let row = sqlx::query(
            "SELECT id, user_id, name, layout_type, camera_positions, is_default, created_at, updated_at FROM grid_layouts WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.as_ref().map(layout_from_row))
    }

    async fn create(&self, layout: &GridLayout) -> Result<(), DomainError> {
        let positions_json = serde_json::to_value(&layout.camera_positions).unwrap_or_default();
        sqlx::query(
            "INSERT INTO grid_layouts (id, user_id, name, layout_type, camera_positions, is_default, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(layout.id)
        .bind(&layout.user_id)
        .bind(&layout.name)
        .bind(layout.layout_type.to_string())
        .bind(positions_json)
        .bind(layout.is_default)
        .bind(layout.created_at)
        .bind(layout.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, layout: &GridLayout) -> Result<(), DomainError> {
        let positions_json = serde_json::to_value(&layout.camera_positions).unwrap_or_default();
        sqlx::query(
            "UPDATE grid_layouts SET name = $2, layout_type = $3, camera_positions = $4, is_default = $5, updated_at = $6 WHERE id = $1"
        )
        .bind(layout.id)
        .bind(&layout.name)
        .bind(layout.layout_type.to_string())
        .bind(positions_json)
        .bind(layout.is_default)
        .bind(layout.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM grid_layouts WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

fn layout_from_row(row: &sqlx::postgres::PgRow) -> GridLayout {
    let positions_json: serde_json::Value = row.get("camera_positions");
    let camera_positions: Vec<CameraPosition> = serde_json::from_value(positions_json).unwrap_or_default();

    GridLayout {
        id: row.get("id"),
        user_id: row.get("user_id"),
        name: row.get("name"),
        layout_type: row.get::<String, _>("layout_type").parse().unwrap_or(LayoutType::Grid),
        camera_positions,
        is_default: row.get("is_default"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
