use async_trait::async_trait;
use crate::errors::DomainError;

/// Represents a raw frame from the camera (video NAL unit or audio frame)
#[derive(Debug, Clone)]
pub enum MediaFrame {
    Video {
        data: Vec<u8>,
        timestamp_us: i64,
        is_keyframe: bool,
    },
    Audio {
        data: Vec<u8>,
        timestamp_us: i64,
        codec: AudioCodec,
    },
}

#[derive(Debug, Clone)]
pub enum AudioCodec {
    Aac,
    G711Alaw,
    G711Mulaw,
    Opus,
}

/// Unified interface for all camera protocol adapters.
/// Each protocol (RTSP, ONVIF, MJPEG, etc.) implements this trait.
#[async_trait]
pub trait StreamIngester: Send + Sync {
    /// Start the connection to the camera and begin receiving frames.
    async fn connect(&mut self) -> Result<(), DomainError>;

    /// Receive the next media frame. Returns None when stream ends.
    async fn next_frame(&mut self) -> Result<Option<MediaFrame>, DomainError>;

    /// Disconnect from the camera.
    async fn disconnect(&mut self) -> Result<(), DomainError>;

    /// Check if the connection is alive.
    fn is_connected(&self) -> bool;

    /// Get stream metadata (codec info, resolution, etc.)
    fn stream_info(&self) -> Option<StreamInfo>;
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub video_codec: String,
    pub audio_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
}
