use open_nvr_domain::entities::*;
use open_nvr_domain::ports::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

const SEGMENT_DURATION: Duration = Duration::from_secs(300); // 5 minutes

/// Frame data that can be sent to live viewers
#[derive(Debug, Clone)]
pub struct LiveFrame {
    pub camera_id: uuid::Uuid,
    pub data: Vec<u8>,
    pub is_keyframe: bool,
    pub timestamp_us: i64,
    pub is_audio: bool,
}

/// Recording worker for a single camera.
/// Connects via RTSP, fans out frames to:
/// 1. Recording (buffer -> segment -> MinIO)
/// 2. Live broadcast channel (for WebSocket relay)
pub async fn camera_recording_task(
    camera: Camera,
    mut ingester: Box<dyn StreamIngester>,
    storage: Arc<dyn ObjectStorage>,
    camera_repo: Arc<dyn CameraRepository>,
    recording_repo: Arc<dyn RecordingRepository>,
    segment_repo: Arc<dyn RecordingSegmentRepository>,
    live_tx: broadcast::Sender<LiveFrame>,
) {
    let camera_id = camera.id;
    let mut backoff = Duration::from_secs(0);
    let max_backoff = Duration::from_secs(camera.max_backoff_secs());
    let frame_timeout = Duration::from_secs(camera.frame_timeout_secs());

    loop {
        // Set camera status to connecting
        let _ = camera_repo
            .update_status(camera_id, CameraStatus::Connecting)
            .await;

        // Connect
        match ingester.connect().await {
            Ok(()) => {
                info!(camera_id = %camera_id, "Camera connected, starting recording");
                let _ = camera_repo
                    .update_status(camera_id, CameraStatus::Online)
                    .await;
                backoff = Duration::from_secs(0);

                // Create new recording
                let recording = Recording::new_continuous(camera_id, camera.audio_capable);
                if let Err(e) = recording_repo.create(&recording).await {
                    error!(camera_id = %camera_id, error = %e, "Failed to create recording");
                    continue;
                }

                // Record until disconnection
                let result = recording_loop(
                    &camera,
                    &mut *ingester,
                    &storage,
                    &segment_repo,
                    &recording,
                    &live_tx,
                    frame_timeout,
                )
                .await;

                // Complete recording
                let mut rec = recording;
                rec.complete();
                let _ = recording_repo.update(&rec).await;

                if let Err(e) = result {
                    warn!(camera_id = %camera_id, error = %e, "Recording ended with error");
                }
            }
            Err(e) => {
                error!(camera_id = %camera_id, error = %e, "Failed to connect");
            }
        }

        // Disconnect and set offline
        let _ = ingester.disconnect().await;
        let _ = camera_repo
            .update_status(camera_id, CameraStatus::Offline)
            .await;

        // Check if recording is disabled
        if camera.recording_mode == RecordingMode::Disabled {
            info!(camera_id = %camera_id, "Recording disabled, stopping");
            break;
        }

        // Exponential backoff
        if backoff.is_zero() {
            backoff = Duration::from_secs(1);
        } else {
            backoff = std::cmp::min(backoff * 2, max_backoff);
        }

        info!(camera_id = %camera_id, backoff_secs = backoff.as_secs(), "Reconnecting after backoff");
        tokio::time::sleep(backoff).await;
    }
}

async fn recording_loop(
    camera: &Camera,
    ingester: &mut dyn StreamIngester,
    storage: &Arc<dyn ObjectStorage>,
    segment_repo: &Arc<dyn RecordingSegmentRepository>,
    recording: &Recording,
    live_tx: &broadcast::Sender<LiveFrame>,
    frame_timeout: Duration,
) -> Result<(), String> {
    let mut segment_buffer: Vec<u8> = Vec::with_capacity(5 * 1024 * 1024);
    let mut segment_start = Instant::now();
    let mut segment_seq = 0;
    let mut last_frame_time = Instant::now();

    loop {
        // Check frame timeout
        if last_frame_time.elapsed() > frame_timeout {
            return Err("Frame timeout - no frames received".into());
        }

        // Get next frame with timeout
        let frame = tokio::time::timeout(frame_timeout, ingester.next_frame()).await;

        match frame {
            Ok(Ok(Some(media_frame))) => {
                last_frame_time = Instant::now();

                match &media_frame {
                    MediaFrame::Video {
                        data,
                        is_keyframe,
                        timestamp_us,
                    } => {
                        if data.is_empty() {
                            continue;
                        }

                        // Broadcast to live viewers
                        let _ = live_tx.send(LiveFrame {
                            camera_id: camera.id,
                            data: data.clone(),
                            is_keyframe: *is_keyframe,
                            timestamp_us: *timestamp_us,
                            is_audio: false,
                        });

                        // Buffer for recording segment
                        segment_buffer.extend_from_slice(data);
                    }
                    MediaFrame::Audio {
                        data, timestamp_us, ..
                    } => {
                        // Broadcast audio to live viewers
                        let _ = live_tx.send(LiveFrame {
                            camera_id: camera.id,
                            data: data.clone(),
                            is_keyframe: false,
                            timestamp_us: *timestamp_us,
                            is_audio: true,
                        });

                        // Buffer audio for recording
                        segment_buffer.extend_from_slice(data);
                    }
                }

                // Check if segment duration reached
                if segment_start.elapsed() >= SEGMENT_DURATION {
                    // Flush segment to storage
                    flush_segment(
                        camera.id,
                        recording.id,
                        segment_seq,
                        &segment_buffer,
                        storage,
                        segment_repo,
                    )
                    .await;

                    segment_buffer.clear();
                    segment_start = Instant::now();
                    segment_seq += 1;
                }
            }
            Ok(Ok(None)) => {
                // Stream ended
                if !segment_buffer.is_empty() {
                    flush_segment(
                        camera.id,
                        recording.id,
                        segment_seq,
                        &segment_buffer,
                        storage,
                        segment_repo,
                    )
                    .await;
                }
                return Err("Stream ended".into());
            }
            Ok(Err(e)) => {
                if !segment_buffer.is_empty() {
                    flush_segment(
                        camera.id,
                        recording.id,
                        segment_seq,
                        &segment_buffer,
                        storage,
                        segment_repo,
                    )
                    .await;
                }
                return Err(format!("Stream error: {}", e));
            }
            Err(_) => {
                // Timeout
                if !segment_buffer.is_empty() {
                    flush_segment(
                        camera.id,
                        recording.id,
                        segment_seq,
                        &segment_buffer,
                        storage,
                        segment_repo,
                    )
                    .await;
                }
                return Err("Frame timeout".into());
            }
        }
    }
}

async fn flush_segment(
    camera_id: uuid::Uuid,
    recording_id: uuid::Uuid,
    seq: i32,
    data: &[u8],
    storage: &Arc<dyn ObjectStorage>,
    segment_repo: &Arc<dyn RecordingSegmentRepository>,
) {
    let storage_key = RecordingSegment::storage_key_for(camera_id, chrono::Utc::now(), seq);

    // Upload to MinIO
    match storage.put_object(&storage_key, data, "video/mp4").await {
        Ok(()) => {
            info!(
                camera_id = %camera_id,
                segment = seq,
                size = data.len(),
                key = %storage_key,
                "Segment uploaded"
            );

            // Create segment record
            let mut segment = RecordingSegment::new(recording_id, seq, storage_key);
            segment.size_bytes = data.len() as i64;
            segment.end_time = Some(chrono::Utc::now());

            if let Err(e) = segment_repo.create(&segment).await {
                error!(error = %e, "Failed to create segment record");
            }
        }
        Err(e) => {
            error!(camera_id = %camera_id, error = %e, "Failed to upload segment");
        }
    }
}
