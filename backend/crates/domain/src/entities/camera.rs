use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolType {
    Rtsp,
    Onvif,
    Mjpeg,
    Rtmp,
    Hls,
    P2p,
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rtsp => write!(f, "rtsp"),
            Self::Onvif => write!(f, "onvif"),
            Self::Mjpeg => write!(f, "mjpeg"),
            Self::Rtmp => write!(f, "rtmp"),
            Self::Hls => write!(f, "hls"),
            Self::P2p => write!(f, "p2p"),
        }
    }
}

impl std::str::FromStr for ProtocolType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rtsp" => Ok(Self::Rtsp),
            "onvif" => Ok(Self::Onvif),
            "mjpeg" => Ok(Self::Mjpeg),
            "rtmp" => Ok(Self::Rtmp),
            "hls" => Ok(Self::Hls),
            "p2p" => Ok(Self::P2p),
            _ => Err(format!("Unknown protocol type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CameraStatus {
    Online,
    Offline,
    Error,
    Connecting,
}

impl std::fmt::Display for CameraStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => write!(f, "online"),
            Self::Offline => write!(f, "offline"),
            Self::Error => write!(f, "error"),
            Self::Connecting => write!(f, "connecting"),
        }
    }
}

impl std::str::FromStr for CameraStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "online" => Ok(Self::Online),
            "offline" => Ok(Self::Offline),
            "error" => Ok(Self::Error),
            "connecting" => Ok(Self::Connecting),
            _ => Err(format!("Unknown camera status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Ethernet,
    Wifi,
}

impl std::fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ethernet => write!(f, "ethernet"),
            Self::Wifi => write!(f, "wifi"),
        }
    }
}

impl std::str::FromStr for ConnectionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ethernet" => Ok(Self::Ethernet),
            "wifi" => Ok(Self::Wifi),
            _ => Err(format!("Unknown connection type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    Continuous,
    Motion,
    Disabled,
}

impl std::fmt::Display for RecordingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continuous => write!(f, "continuous"),
            Self::Motion => write!(f, "motion"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

impl std::str::FromStr for RecordingMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "continuous" => Ok(Self::Continuous),
            "motion" => Ok(Self::Motion),
            "disabled" => Ok(Self::Disabled),
            _ => Err(format!("Unknown recording mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub id: Uuid,
    pub name: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub protocol_type: ProtocolType,
    pub stream_url: String,
    pub sub_stream_url: Option<String>,
    pub onvif_url: Option<String>,
    pub credentials_encrypted: Option<Vec<u8>>,
    pub ptz_capable: bool,
    pub audio_capable: bool,
    pub group_id: Option<Uuid>,
    pub status: CameraStatus,
    pub connection_type: ConnectionType,
    pub recording_mode: RecordingMode,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Camera {
    pub fn new(name: String, protocol_type: ProtocolType, stream_url: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            brand: None,
            model: None,
            protocol_type,
            stream_url,
            sub_stream_url: None,
            onvif_url: None,
            credentials_encrypted: None,
            ptz_capable: false,
            audio_capable: false,
            group_id: None,
            status: CameraStatus::Offline,
            connection_type: ConnectionType::Ethernet,
            recording_mode: RecordingMode::Continuous,
            config: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_wifi(&self) -> bool {
        self.connection_type == ConnectionType::Wifi
    }

    pub fn frame_timeout_secs(&self) -> u64 {
        if self.is_wifi() { 15 } else { 10 }
    }

    pub fn max_backoff_secs(&self) -> u64 {
        if self.is_wifi() { 90 } else { 60 }
    }
}
