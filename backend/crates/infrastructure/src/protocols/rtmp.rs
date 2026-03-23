use async_trait::async_trait;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::{MediaFrame, StreamInfo, StreamIngester};
use tracing::warn;

/// RTMP ingester for push-based cameras.
/// Some cheap Chinese cameras push RTMP to a server rather than serving RTSP.
/// This adapter acts as a simple RTMP receiver on a configured port.
///
/// NOTE: Full RTMP implementation requires a dedicated RTMP server.
/// This is a placeholder that will be expanded with rtmp-server crate.
pub struct RtmpIngester {
    url: String,
    connected: bool,
}

impl RtmpIngester {
    pub fn new(url: &str, _username: Option<&str>, _password: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            connected: false,
        }
    }
}

#[async_trait]
impl StreamIngester for RtmpIngester {
    async fn connect(&mut self) -> Result<(), DomainError> {
        warn!(url = %self.url, "RTMP ingester is a placeholder - full implementation pending");
        // TODO: Implement RTMP pull/push server
        // For now, return an error indicating RTMP is not yet fully supported
        Err(DomainError::CameraConnection(
            "RTMP protocol support is in development. Use RTSP if available.".into()
        ))
    }

    async fn next_frame(&mut self) -> Result<Option<MediaFrame>, DomainError> {
        Err(DomainError::Stream("RTMP not yet implemented".into()))
    }

    async fn disconnect(&mut self) -> Result<(), DomainError> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn stream_info(&self) -> Option<StreamInfo> {
        None
    }
}
