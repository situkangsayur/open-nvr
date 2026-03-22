use async_trait::async_trait;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::{AudioCodec, MediaFrame, StreamInfo, StreamIngester};
use retina::client::{SessionGroup, SetupOptions};
use retina::codec::CodecItem;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

pub struct RtspIngester {
    url: String,
    username: Option<String>,
    password: Option<String>,
    session: Arc<Mutex<Option<RtspSession>>>,
    connected: bool,
    stream_info: Option<StreamInfo>,
}

struct RtspSession {
    demuxed: retina::client::Demuxed,
}

impl RtspIngester {
    pub fn new(url: &str, username: Option<&str>, password: Option<&str>) -> Self {
        Self {
            url: url.to_string(),
            username: username.map(String::from),
            password: password.map(String::from),
            session: Arc::new(Mutex::new(None)),
            connected: false,
            stream_info: None,
        }
    }
}

#[async_trait]
impl StreamIngester for RtspIngester {
    async fn connect(&mut self) -> Result<(), DomainError> {
        info!(url = %self.url, "Connecting to RTSP camera");

        let parsed_url = url::Url::parse(&self.url)
            .map_err(|e| DomainError::CameraConnection(format!("Invalid RTSP URL: {}", e)))?;

        let creds = match (&self.username, &self.password) {
            (Some(u), Some(p)) => Some(retina::client::Credentials {
                username: u.clone(),
                password: p.clone(),
            }),
            _ => None,
        };

        let session_group = Arc::new(SessionGroup::default());
        let mut session = retina::client::Session::describe(
            parsed_url,
            retina::client::SessionOptions::default()
                .creds(creds)
                .session_group(session_group),
        )
        .await
        .map_err(|e| DomainError::CameraConnection(format!("RTSP DESCRIBE failed: {}", e)))?;

        // Setup video stream (usually index 0)
        let video_stream_i = {
            let mut found = None;
            for (i, stream) in session.streams().iter().enumerate() {
                if stream.media() == "video" {
                    found = Some(i);
                    break;
                }
            }
            found.ok_or_else(|| DomainError::CameraConnection("No video stream found".into()))?
        };

        session
            .setup(video_stream_i, SetupOptions::default())
            .await
            .map_err(|e| {
                DomainError::CameraConnection(format!("RTSP SETUP video failed: {}", e))
            })?;

        // Try to setup audio stream if available
        let mut has_audio = false;
        for (i, stream) in session.streams().iter().enumerate() {
            if stream.media() == "audio" {
                match session.setup(i, SetupOptions::default()).await {
                    Ok(_) => {
                        has_audio = true;
                        info!("Audio stream setup successful");
                    }
                    Err(e) => {
                        warn!("Audio stream setup failed (continuing without audio): {}", e);
                    }
                }
                break;
            }
        }

        let _video_params = session.streams()[video_stream_i].parameters();
        self.stream_info = Some(StreamInfo {
            video_codec: "h264".to_string(),
            audio_codec: if has_audio {
                Some("aac".to_string())
            } else {
                None
            },
            width: None,
            height: None,
            fps: None,
        });

        let demuxed = session
            .play(retina::client::PlayOptions::default())
            .await
            .map_err(|e| DomainError::CameraConnection(format!("RTSP PLAY failed: {}", e)))?
            .demuxed()
            .map_err(|e| DomainError::CameraConnection(format!("RTSP demux failed: {}", e)))?;

        *self.session.lock().await = Some(RtspSession { demuxed });
        self.connected = true;

        info!(url = %self.url, "RTSP connection established");
        Ok(())
    }

    async fn next_frame(&mut self) -> Result<Option<MediaFrame>, DomainError> {
        use futures_util::StreamExt;

        let mut session_guard = self.session.lock().await;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| DomainError::Stream("Not connected".into()))?;

        match session.demuxed.next().await {
            Some(Ok(item)) => match item {
                CodecItem::VideoFrame(frame) => {
                    let is_key = frame.is_random_access_point();
                    let data = frame.data().to_vec();
                    let timestamp = frame.timestamp();
                    debug!(
                        size = data.len(),
                        is_keyframe = is_key,
                        "Received video frame"
                    );
                    Ok(Some(MediaFrame::Video {
                        data,
                        timestamp_us: timestamp.elapsed() as i64,
                        is_keyframe: is_key,
                    }))
                }
                CodecItem::AudioFrame(frame) => {
                    let data = frame.data().to_vec();
                    let timestamp = frame.timestamp();
                    Ok(Some(MediaFrame::Audio {
                        data,
                        timestamp_us: timestamp.elapsed() as i64,
                        codec: AudioCodec::Aac,
                    }))
                }
                _ => {
                    // Skip other items (e.g., MessageFrame)
                    Ok(Some(MediaFrame::Video {
                        data: Vec::new(),
                        timestamp_us: 0,
                        is_keyframe: false,
                    }))
                }
            },
            Some(Err(e)) => {
                error!(error = %e, "RTSP stream error");
                self.connected = false;
                Err(DomainError::Stream(format!("RTSP error: {}", e)))
            }
            None => {
                self.connected = false;
                Ok(None)
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), DomainError> {
        *self.session.lock().await = None;
        self.connected = false;
        info!(url = %self.url, "RTSP disconnected");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn stream_info(&self) -> Option<StreamInfo> {
        self.stream_info.clone()
    }
}
