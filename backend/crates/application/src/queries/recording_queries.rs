use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::*;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct RecordingResponse {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub recording_type: String,
    pub has_audio: bool,
    pub total_size: i64,
    pub status: String,
    pub duration_secs: Option<i64>,
}

impl From<Recording> for RecordingResponse {
    fn from(r: Recording) -> Self {
        let duration = r.duration_secs();
        Self {
            id: r.id,
            camera_id: r.camera_id,
            start_time: r.start_time,
            end_time: r.end_time,
            recording_type: r.recording_type.to_string(),
            has_audio: r.has_audio,
            total_size: r.total_size,
            status: r.status.to_string(),
            duration_secs: duration,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SegmentResponse {
    pub id: Uuid,
    pub sequence_number: i32,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<i32>,
    pub size_bytes: i64,
    pub download_url: Option<String>,
}

pub struct RecordingQueryService {
    recording_repo: Arc<dyn RecordingRepository>,
    segment_repo: Arc<dyn RecordingSegmentRepository>,
    storage: Option<Arc<dyn ObjectStorage>>,
}

impl RecordingQueryService {
    pub fn new(
        recording_repo: Arc<dyn RecordingRepository>,
        segment_repo: Arc<dyn RecordingSegmentRepository>,
        storage: Option<Arc<dyn ObjectStorage>>,
    ) -> Self {
        Self {
            recording_repo,
            segment_repo,
            storage,
        }
    }

    pub async fn get_recording(&self, id: Uuid) -> Result<RecordingResponse, DomainError> {
        let recording = self
            .recording_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NotFound {
                entity_type: "recording".into(),
                id,
            })?;
        Ok(recording.into())
    }

    pub async fn list_by_camera(
        &self,
        camera_id: Uuid,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<RecordingResponse>, DomainError> {
        let recordings = self.recording_repo.find_by_camera(camera_id, start, end).await?;
        Ok(recordings.into_iter().map(|r| r.into()).collect())
    }

    pub async fn get_segments_with_urls(
        &self,
        recording_id: Uuid,
    ) -> Result<Vec<SegmentResponse>, DomainError> {
        let segments = self.segment_repo.find_by_recording(recording_id).await?;

        let mut responses = Vec::new();
        for seg in segments {
            let url = match &self.storage {
                Some(s) => s.presigned_url(&seg.storage_key, 3600).await.ok(),
                None => None,
            };

            responses.push(SegmentResponse {
                id: seg.id,
                sequence_number: seg.sequence_number,
                start_time: seg.start_time,
                end_time: seg.end_time,
                duration_ms: seg.duration_ms,
                size_bytes: seg.size_bytes,
                download_url: url,
            });
        }

        Ok(responses)
    }
}
