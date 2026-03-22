use async_trait::async_trait;
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::CameraRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgCameraRepository {
    pool: PgPool,
}

impl PgCameraRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn camera_from_row(r: sqlx::postgres::PgRow) -> Camera {
    let row = CameraRow {
        id: r.get("id"),
        name: r.get("name"),
        brand: r.get("brand"),
        model: r.get("model"),
        protocol_type: r.get("protocol_type"),
        stream_url: r.get("stream_url"),
        sub_stream_url: r.get("sub_stream_url"),
        onvif_url: r.get("onvif_url"),
        credentials_encrypted: r.get("credentials_encrypted"),
        ptz_capable: r.get("ptz_capable"),
        audio_capable: r.get("audio_capable"),
        group_id: r.get("group_id"),
        status: r.get("status"),
        connection_type: r.get("connection_type"),
        recording_mode: r.get("recording_mode"),
        config: r.get("config"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    };
    row.into()
}

#[async_trait]
impl CameraRepository for PgCameraRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Camera>, DomainError> {
        let row = sqlx::query(
            r#"SELECT id, name, brand, model, protocol_type, stream_url, sub_stream_url,
                      onvif_url, credentials_encrypted, ptz_capable, audio_capable,
                      group_id, status, connection_type, recording_mode,
                      config, created_at, updated_at
               FROM cameras WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(row.map(camera_from_row))
    }

    async fn find_all(&self) -> Result<Vec<Camera>, DomainError> {
        let rows = sqlx::query(
            r#"SELECT id, name, brand, model, protocol_type, stream_url, sub_stream_url,
                      onvif_url, credentials_encrypted, ptz_capable, audio_capable,
                      group_id, status, connection_type, recording_mode,
                      config, created_at, updated_at
               FROM cameras ORDER BY name"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(camera_from_row).collect())
    }

    async fn find_by_group(&self, group_id: Uuid) -> Result<Vec<Camera>, DomainError> {
        let rows = sqlx::query(
            r#"SELECT id, name, brand, model, protocol_type, stream_url, sub_stream_url,
                      onvif_url, credentials_encrypted, ptz_capable, audio_capable,
                      group_id, status, connection_type, recording_mode,
                      config, created_at, updated_at
               FROM cameras WHERE group_id = $1 ORDER BY name"#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(camera_from_row).collect())
    }

    async fn find_by_status(&self, status: CameraStatus) -> Result<Vec<Camera>, DomainError> {
        let status_str = status.to_string();
        let rows = sqlx::query(
            r#"SELECT id, name, brand, model, protocol_type, stream_url, sub_stream_url,
                      onvif_url, credentials_encrypted, ptz_capable, audio_capable,
                      group_id, status, connection_type, recording_mode,
                      config, created_at, updated_at
               FROM cameras WHERE status = $1 ORDER BY name"#,
        )
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(camera_from_row).collect())
    }

    async fn create(&self, camera: &Camera) -> Result<(), DomainError> {
        sqlx::query(
            r#"INSERT INTO cameras (id, name, brand, model, protocol_type, stream_url,
                sub_stream_url, onvif_url, credentials_encrypted, ptz_capable, audio_capable,
                group_id, status, connection_type, recording_mode, config, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"#,
        )
        .bind(camera.id)
        .bind(&camera.name)
        .bind(&camera.brand)
        .bind(&camera.model)
        .bind(camera.protocol_type.to_string())
        .bind(&camera.stream_url)
        .bind(&camera.sub_stream_url)
        .bind(&camera.onvif_url)
        .bind(&camera.credentials_encrypted)
        .bind(camera.ptz_capable)
        .bind(camera.audio_capable)
        .bind(camera.group_id)
        .bind(camera.status.to_string())
        .bind(camera.connection_type.to_string())
        .bind(camera.recording_mode.to_string())
        .bind(&camera.config)
        .bind(camera.created_at)
        .bind(camera.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, camera: &Camera) -> Result<(), DomainError> {
        sqlx::query(
            r#"UPDATE cameras SET name = $2, brand = $3, model = $4, protocol_type = $5,
                stream_url = $6, sub_stream_url = $7, onvif_url = $8,
                credentials_encrypted = $9, ptz_capable = $10, audio_capable = $11,
                group_id = $12, status = $13, connection_type = $14, recording_mode = $15,
                config = $16, updated_at = $17
               WHERE id = $1"#,
        )
        .bind(camera.id)
        .bind(&camera.name)
        .bind(&camera.brand)
        .bind(&camera.model)
        .bind(camera.protocol_type.to_string())
        .bind(&camera.stream_url)
        .bind(&camera.sub_stream_url)
        .bind(&camera.onvif_url)
        .bind(&camera.credentials_encrypted)
        .bind(camera.ptz_capable)
        .bind(camera.audio_capable)
        .bind(camera.group_id)
        .bind(camera.status.to_string())
        .bind(camera.connection_type.to_string())
        .bind(camera.recording_mode.to_string())
        .bind(&camera.config)
        .bind(camera.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM cameras WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update_status(&self, id: Uuid, status: CameraStatus) -> Result<(), DomainError> {
        sqlx::query("UPDATE cameras SET status = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(status.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

// Internal row struct for mapping
struct CameraRow {
    id: Uuid,
    name: String,
    brand: Option<String>,
    model: Option<String>,
    protocol_type: String,
    stream_url: String,
    sub_stream_url: Option<String>,
    onvif_url: Option<String>,
    credentials_encrypted: Option<Vec<u8>>,
    ptz_capable: bool,
    audio_capable: bool,
    group_id: Option<Uuid>,
    status: String,
    connection_type: String,
    recording_mode: String,
    config: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CameraRow> for Camera {
    fn from(row: CameraRow) -> Self {
        Camera {
            id: row.id,
            name: row.name,
            brand: row.brand,
            model: row.model,
            protocol_type: row.protocol_type.parse().unwrap_or(ProtocolType::Rtsp),
            stream_url: row.stream_url,
            sub_stream_url: row.sub_stream_url,
            onvif_url: row.onvif_url,
            credentials_encrypted: row.credentials_encrypted,
            ptz_capable: row.ptz_capable,
            audio_capable: row.audio_capable,
            group_id: row.group_id,
            status: row.status.parse().unwrap_or(CameraStatus::Offline),
            connection_type: row.connection_type.parse().unwrap_or(ConnectionType::Ethernet),
            recording_mode: row.recording_mode.parse().unwrap_or(RecordingMode::Continuous),
            config: row.config,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
