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
    /// Whether a camera login is stored. The login itself is never returned.
    pub has_credentials: bool,
    pub group_id: Option<Uuid>,
    pub status: String,
    pub connection_type: String,
    pub recording_mode: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CreateCameraRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 255 {
            return Err("Camera name must be 1-255 characters".into());
        }
        if self.stream_url.trim().is_empty() || self.stream_url.len() > 2048 {
            return Err("Stream URL must be 1-2048 characters".into());
        }
        // Prevent SSRF - only allow known protocols
        let valid_schemes = ["rtsp://", "rtsps://", "http://", "https://", "rtmp://"];
        if !valid_schemes.iter().any(|s| self.stream_url.starts_with(s)) {
            return Err("Stream URL must start with rtsp://, rtsps://, http://, https://, or rtmp://".into());
        }
        // Prevent internal network scanning via stream URL
        if self.stream_url.contains("localhost") || self.stream_url.contains("127.0.0.1") || self.stream_url.contains("0.0.0.0") {
            return Err("Stream URL cannot point to localhost".into());
        }
        if let Some(ref url) = self.onvif_url {
            if url.len() > 2048 {
                return Err("ONVIF URL too long".into());
            }
        }
        if let Some(ref brand) = self.brand {
            if brand.len() > 100 {
                return Err("Brand must be under 100 characters".into());
            }
        }
        if let Some(ref user) = self.username {
            if user.len() > 255 {
                return Err("Username too long".into());
            }
        }
        if let Some(ref pass) = self.password {
            if pass.len() > 255 {
                return Err("Password too long".into());
            }
        }
        Ok(())
    }
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
            has_credentials: c.credentials_encrypted.is_some(),
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

impl CreateGroupRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 255 {
            return Err("Group name must be 1-255 characters".into());
        }
        if let Some(ref desc) = self.description {
            if desc.len() > 1000 {
                return Err("Description must be under 1000 characters".into());
            }
        }
        Ok(())
    }
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

impl ScanNetworkRequest {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref subnets) = self.subnets {
            if subnets.len() > 10 {
                return Err("Maximum 10 subnets per scan".into());
            }
            for subnet in subnets {
                if subnet.len() > 50 {
                    return Err("Subnet too long".into());
                }
                // Basic CIDR validation
                if !subnet.contains('/') {
                    return Err(format!("Invalid CIDR notation: {}", subnet));
                }
            }
        }
        if let Some(timeout) = self.timeout_secs {
            if timeout > 120 {
                return Err("Timeout must be 120 seconds or less".into());
            }
        }
        Ok(())
    }
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
