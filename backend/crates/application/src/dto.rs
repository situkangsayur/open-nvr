use chrono::{DateTime, Utc};
use open_nvr_domain::entities::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// === Camera DTOs ===

#[derive(Debug, Deserialize)]
pub struct CreateCameraRequest {
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub protocol_type: String,
    pub stream_url: String,
    pub sub_stream_url: Option<String>,
    pub onvif_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ptz_capable: Option<bool>,
    pub audio_capable: Option<bool>,
    pub group_id: Option<Uuid>,
    pub connection_type: Option<String>,
    pub recording_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCameraRequest {
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub stream_url: Option<String>,
    pub sub_stream_url: Option<String>,
    pub onvif_url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ptz_capable: Option<bool>,
    pub audio_capable: Option<bool>,
    pub group_id: Option<Uuid>,
    pub connection_type: Option<String>,
    pub recording_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CameraResponse {
    pub id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub protocol_type: String,
    pub stream_url: String,
    pub sub_stream_url: Option<String>,
    pub onvif_url: Option<String>,
    pub ptz_capable: bool,
    pub audio_capable: bool,
    pub group_id: Option<Uuid>,
    pub status: String,
    pub connection_type: String,
    pub recording_mode: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Camera> for CameraResponse {
    fn from(c: Camera) -> Self {
        Self {
            id: c.id,
            name: c.name,
            brand: c.brand,
            model: c.model,
            protocol_type: c.protocol_type.to_string(),
            stream_url: c.stream_url,
            sub_stream_url: c.sub_stream_url,
            onvif_url: c.onvif_url,
            ptz_capable: c.ptz_capable,
            audio_capable: c.audio_capable,
            group_id: c.group_id,
            status: c.status.to_string(),
            connection_type: c.connection_type.to_string(),
            recording_mode: c.recording_mode.to_string(),
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

// === Camera Group DTOs ===

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GroupResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<CameraGroup> for GroupResponse {
    fn from(g: CameraGroup) -> Self {
        Self {
            id: g.id,
            name: g.name,
            description: g.description,
            created_at: g.created_at,
        }
    }
}

// === Audit DTOs ===

#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLog> for AuditLogResponse {
    fn from(a: AuditLog) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            user_email: a.user_email,
            action: a.action,
            resource_type: a.resource_type,
            resource_id: a.resource_id,
            details: a.details,
            ip_address: a.ip_address.map(|ip| ip.to_string()),
            created_at: a.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NetworkEventResponse {
    pub id: Uuid,
    pub event_type: String,
    pub source_ip: Option<String>,
    pub source_mac: Option<String>,
    pub target_resource: Option<String>,
    pub severity: String,
    pub details: serde_json::Value,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
}

impl From<NetworkEvent> for NetworkEventResponse {
    fn from(e: NetworkEvent) -> Self {
        Self {
            id: e.id,
            event_type: e.event_type.to_string(),
            source_ip: e.source_ip.map(|ip| ip.to_string()),
            source_mac: e.source_mac,
            target_resource: e.target_resource,
            severity: e.severity.to_string(),
            details: e.details,
            resolved: e.resolved,
            created_at: e.created_at,
        }
    }
}

// === Discovery DTOs ===

#[derive(Debug, Deserialize)]
pub struct ScanNetworkRequest {
    pub subnets: Option<Vec<String>>,
    pub timeout_secs: Option<u64>,
}

// === Storage DTOs ===

#[derive(Debug, Serialize)]
pub struct StorageEstimate {
    pub total_bytes: u64,
    pub formatted: String,
    pub cameras: u32,
    pub bitrate_mbps: f64,
    pub retention_days: u32,
    pub motion_ratio: f64,
}

#[derive(Debug, Deserialize)]
pub struct StorageEstimateRequest {
    pub cameras: u32,
    pub bitrate_mbps: f64,
    pub retention_days: u32,
    pub motion_ratio: Option<f64>,
}
