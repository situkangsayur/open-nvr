use async_trait::async_trait;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::{MediaFrame, StreamInfo, StreamIngester};
use tracing::info;

/// MJPEG stream ingester for HTTP MJPEG cameras (ESP32-CAM, cheap webcams).
/// Reads multipart JPEG frames from an HTTP stream.
pub struct MjpegIngester {
    url: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
    connected: bool,
    response: Option<reqwest::Response>,
    buffer: Vec<u8>,
}

impl MjpegIngester {
    pub fn new(url: &str, username: Option<&str>, password: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            username: username.map(String::from),
            password: password.map(String::from),
            client: reqwest::Client::new(),
            connected: false,
            response: None,
            buffer: Vec::with_capacity(512 * 1024),
        }
    }
}

#[async_trait]
impl StreamIngester for MjpegIngester {
    async fn connect(&mut self) -> Result<(), DomainError> {
        info!(url = %self.url, "Connecting to MJPEG stream");

        let mut request = self.client.get(&self.url);
        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            request = request.basic_auth(user, Some(pass));
        }

        let response = request
            .send()
            .await
            .map_err(|e| DomainError::CameraConnection(format!("MJPEG connect failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(DomainError::CameraConnection(format!(
                "MJPEG HTTP error: {}",
                response.status()
            )));
        }

        self.response = Some(response);
        self.connected = true;
        info!(url = %self.url, "MJPEG stream connected");
        Ok(())
    }

    async fn next_frame(&mut self) -> Result<Option<MediaFrame>, DomainError> {
        let response = self.response.as_mut()
            .ok_or_else(|| DomainError::Stream("Not connected".into()))?;

        // Read chunks until we find a complete JPEG frame
        // MJPEG streams use multipart boundaries: --boundary\r\nContent-Type: image/jpeg\r\n\r\n<data>\r\n
        loop {
            let chunk = response.chunk().await
                .map_err(|e| DomainError::Stream(format!("MJPEG read error: {}", e)))?;

            match chunk {
                Some(data) => {
                    self.buffer.extend_from_slice(&data);

                    // Look for JPEG start (FFD8) and end (FFD9) markers
                    if let Some(frame) = extract_jpeg_frame(&mut self.buffer) {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_micros() as i64;

                        return Ok(Some(MediaFrame::Video {
                            data: frame,
                            timestamp_us: timestamp,
                            is_keyframe: true, // Every MJPEG frame is a keyframe
                        }));
                    }

                    // Prevent buffer from growing too large
                    if self.buffer.len() > 10 * 1024 * 1024 {
                        self.buffer.clear();
                    }
                }
                None => {
                    self.connected = false;
                    return Ok(None);
                }
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), DomainError> {
        self.response = None;
        self.connected = false;
        self.buffer.clear();
        info!(url = %self.url, "MJPEG disconnected");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn stream_info(&self) -> Option<StreamInfo> {
        Some(StreamInfo {
            video_codec: "mjpeg".to_string(),
            audio_codec: None,
            width: None,
            height: None,
            fps: None,
        })
    }
}

/// Extract a complete JPEG frame from the buffer.
/// Looks for FFD8 (start) and FFD9 (end) markers.
fn extract_jpeg_frame(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    // Find JPEG SOI marker (FFD8)
    let start = buffer.windows(2).position(|w| w == [0xFF, 0xD8])?;

    // Find JPEG EOI marker (FFD9) after the start
    let end_search = &buffer[start + 2..];
    let end_offset = end_search.windows(2).position(|w| w == [0xFF, 0xD9])?;
    let end = start + 2 + end_offset + 2; // Include the FFD9 marker

    if end > buffer.len() {
        return None;
    }

    let frame = buffer[start..end].to_vec();

    // Remove consumed data from buffer
    buffer.drain(..end);

    Some(frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_jpeg_frame() {
        let mut buffer = vec![
            0x00, 0x00, // garbage
            0xFF, 0xD8, // SOI
            0x01, 0x02, 0x03, // data
            0xFF, 0xD9, // EOI
            0x00, 0x00, // trailing
        ];

        let frame = extract_jpeg_frame(&mut buffer).unwrap();
        assert_eq!(frame, vec![0xFF, 0xD8, 0x01, 0x02, 0x03, 0xFF, 0xD9]);
        assert_eq!(buffer, vec![0x00, 0x00]); // trailing remains
    }

    #[test]
    fn test_no_complete_frame() {
        let mut buffer = vec![0xFF, 0xD8, 0x01, 0x02]; // No EOI
        assert!(extract_jpeg_frame(&mut buffer).is_none());
    }
}
