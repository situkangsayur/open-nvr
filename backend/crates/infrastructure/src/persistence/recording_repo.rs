use async_trait::async_trait;
use chrono::{DateTime, Utc};
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::{RecordingRepository, RecordingSegmentRepository};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgRecordingRepository {
    pool: PgPool,
}

impl PgRecordingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecordingRepository for PgRecordingRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Recording>, DomainError> {
        let row = sqlx::query(
            "SELECT id, camera_id, start_time, end_time, recording_type, has_audio, total_size, status, created_at FROM recordings WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(|r| recording_from_row(&r)))
    }

    async fn find_by_camera(
        &self,
        camera_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Recording>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, camera_id, start_time, end_time, recording_type, has_audio, total_size, status, created_at FROM recordings WHERE camera_id = $1 AND start_time >= $2 AND start_time <= $3 ORDER BY start_time",
        )
        .bind(camera_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(recording_from_row).collect())
    }

    async fn create(&self, recording: &Recording) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO recordings (id, camera_id, start_time, end_time, recording_type, has_audio, total_size, status, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(recording.id)
        .bind(recording.camera_id)
        .bind(recording.start_time)
        .bind(recording.end_time)
        .bind(recording.recording_type.to_string())
        .bind(recording.has_audio)
        .bind(recording.total_size)
        .bind(recording.status.to_string())
        .bind(recording.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, recording: &Recording) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE recordings SET end_time = $2, total_size = $3, status = $4 WHERE id = $1",
        )
        .bind(recording.id)
        .bind(recording.end_time)
        .bind(recording.total_size)
        .bind(recording.status.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_active_by_camera(
        &self,
        camera_id: Uuid,
    ) -> Result<Option<Recording>, DomainError> {
        let row = sqlx::query(
            "SELECT id, camera_id, start_time, end_time, recording_type, has_audio, total_size, status, created_at FROM recordings WHERE camera_id = $1 AND status = 'recording' ORDER BY start_time DESC LIMIT 1",
        )
        .bind(camera_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(|r| recording_from_row(&r)))
    }
}

fn recording_from_row(row: &sqlx::postgres::PgRow) -> Recording {
    Recording {
        id: row.get("id"),
        camera_id: row.get("camera_id"),
        start_time: row.get("start_time"),
        end_time: row.get("end_time"),
        recording_type: row
            .get::<String, _>("recording_type")
            .parse()
            .unwrap_or(RecordingType::Continuous),
        has_audio: row.get("has_audio"),
        total_size: row.get("total_size"),
        status: row
            .get::<String, _>("status")
            .parse()
            .unwrap_or(RecordingStatus::Recording),
        created_at: row.get("created_at"),
    }
}

pub struct PgRecordingSegmentRepository {
    pool: PgPool,
}

impl PgRecordingSegmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RecordingSegmentRepository for PgRecordingSegmentRepository {
    async fn find_by_recording(
        &self,
        recording_id: Uuid,
    ) -> Result<Vec<RecordingSegment>, DomainError> {
        let rows = sqlx::query(
            "SELECT id, recording_id, sequence_number, storage_key, start_time, end_time, duration_ms, size_bytes, created_at FROM recording_segments WHERE recording_id = $1 ORDER BY sequence_number",
        )
        .bind(recording_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.iter().map(segment_from_row).collect())
    }

    async fn create(&self, segment: &RecordingSegment) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO recording_segments (id, recording_id, sequence_number, storage_key, start_time, end_time, duration_ms, size_bytes, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(segment.id)
        .bind(segment.recording_id)
        .bind(segment.sequence_number)
        .bind(&segment.storage_key)
        .bind(segment.start_time)
        .bind(segment.end_time)
        .bind(segment.duration_ms)
        .bind(segment.size_bytes)
        .bind(segment.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, segment: &RecordingSegment) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE recording_segments SET end_time = $2, duration_ms = $3, size_bytes = $4 WHERE id = $1",
        )
        .bind(segment.id)
        .bind(segment.end_time)
        .bind(segment.duration_ms)
        .bind(segment.size_bytes)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

fn segment_from_row(row: &sqlx::postgres::PgRow) -> RecordingSegment {
    RecordingSegment {
        id: row.get("id"),
        recording_id: row.get("recording_id"),
        sequence_number: row.get("sequence_number"),
        storage_key: row.get("storage_key"),
        start_time: row.get("start_time"),
        end_time: row.get("end_time"),
        duration_ms: row.get("duration_ms"),
        size_bytes: row.get("size_bytes"),
        created_at: row.get("created_at"),
    }
}
