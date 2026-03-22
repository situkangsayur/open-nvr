use async_trait::async_trait;
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::DetectionZoneRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgDetectionZoneRepository {
    pool: PgPool,
}

impl PgDetectionZoneRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DetectionZoneRepository for PgDetectionZoneRepository {
    async fn find_by_camera(&self, camera_id: Uuid) -> Result<Vec<DetectionZone>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, camera_id, name, polygon, detection_types, sensitivity, enabled, created_at, updated_at FROM detection_zones WHERE camera_id = $1 ORDER BY name"
        )
        .bind(camera_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(zone_from_row).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<DetectionZone>, DomainError> {
        let row = sqlx::query(
            "SELECT id, camera_id, name, polygon, detection_types, sensitivity, enabled, created_at, updated_at FROM detection_zones WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.as_ref().map(zone_from_row))
    }

    async fn create(&self, zone: &DetectionZone) -> Result<(), DomainError> {
        let polygon_json = serde_json::to_value(&zone.polygon).unwrap_or_default();
        let types_json = serde_json::to_value(&zone.detection_types).unwrap_or_default();

        sqlx::query(
            "INSERT INTO detection_zones (id, camera_id, name, polygon, detection_types, sensitivity, enabled, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(zone.id)
        .bind(zone.camera_id)
        .bind(&zone.name)
        .bind(polygon_json)
        .bind(types_json)
        .bind(zone.sensitivity)
        .bind(zone.enabled)
        .bind(zone.created_at)
        .bind(zone.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, zone: &DetectionZone) -> Result<(), DomainError> {
        let polygon_json = serde_json::to_value(&zone.polygon).unwrap_or_default();
        let types_json = serde_json::to_value(&zone.detection_types).unwrap_or_default();

        sqlx::query(
            "UPDATE detection_zones SET name = $2, polygon = $3, detection_types = $4, sensitivity = $5, enabled = $6, updated_at = $7 WHERE id = $1"
        )
        .bind(zone.id)
        .bind(&zone.name)
        .bind(polygon_json)
        .bind(types_json)
        .bind(zone.sensitivity)
        .bind(zone.enabled)
        .bind(zone.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM detection_zones WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

fn zone_from_row(row: &sqlx::postgres::PgRow) -> DetectionZone {
    let polygon_json: serde_json::Value = row.get("polygon");
    let polygon: Vec<Point> = serde_json::from_value(polygon_json).unwrap_or_default();

    let types_json: serde_json::Value = row.get("detection_types");
    let detection_types: Vec<DetectionEventType> =
        serde_json::from_value(types_json).unwrap_or_default();

    DetectionZone {
        id: row.get("id"),
        camera_id: row.get("camera_id"),
        name: row.get("name"),
        polygon,
        detection_types,
        sensitivity: row.get("sensitivity"),
        enabled: row.get("enabled"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
