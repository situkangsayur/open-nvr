use async_trait::async_trait;
use chrono::{DateTime, Utc};
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::DetectionEventRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgDetectionEventRepository {
    pool: PgPool,
}

impl PgDetectionEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DetectionEventRepository for PgDetectionEventRepository {
    async fn find_by_camera(
        &self,
        camera_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<DetectionEvent>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, camera_id, zone_id, event_type, confidence, bounding_box, thumbnail_key, metadata, occurred_at, created_at FROM detection_events WHERE camera_id = $1 AND occurred_at >= $2 AND occurred_at <= $3 ORDER BY occurred_at",
        )
        .bind(camera_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(event_from_row).collect())
    }

    async fn find_by_type(
        &self,
        event_type: DetectionEventType,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<DetectionEvent>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, camera_id, zone_id, event_type, confidence, bounding_box, thumbnail_key, metadata, occurred_at, created_at FROM detection_events WHERE event_type = $1 AND occurred_at >= $2 AND occurred_at <= $3 ORDER BY occurred_at",
        )
        .bind(event_type.to_string())
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(event_from_row).collect())
    }

    async fn create(&self, event: &DetectionEvent) -> Result<(), DomainError> {
        let bbox_json = event
            .bounding_box
            .as_ref()
            .map(|b| serde_json::to_value(b).unwrap_or_default());
        sqlx::query(
            "INSERT INTO detection_events (id, camera_id, zone_id, event_type, confidence, bounding_box, thumbnail_key, metadata, occurred_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(event.id)
        .bind(event.camera_id)
        .bind(event.zone_id)
        .bind(event.event_type.to_string())
        .bind(event.confidence)
        .bind(bbox_json)
        .bind(&event.thumbnail_key)
        .bind(&event.metadata)
        .bind(event.occurred_at)
        .bind(event.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

fn event_from_row(row: &sqlx::postgres::PgRow) -> DetectionEvent {
    let bbox: Option<serde_json::Value> = row.get("bounding_box");
    let bounding_box = bbox.and_then(|v| serde_json::from_value(v).ok());

    DetectionEvent {
        id: row.get("id"),
        camera_id: row.get("camera_id"),
        zone_id: row.get("zone_id"),
        event_type: row
            .get::<String, _>("event_type")
            .parse()
            .unwrap_or(DetectionEventType::Unknown),
        confidence: row.get("confidence"),
        bounding_box,
        thumbnail_key: row.get("thumbnail_key"),
        metadata: row
            .get::<Option<serde_json::Value>, _>("metadata")
            .unwrap_or_default(),
        occurred_at: row.get("occurred_at"),
        created_at: row.get("created_at"),
    }
}
