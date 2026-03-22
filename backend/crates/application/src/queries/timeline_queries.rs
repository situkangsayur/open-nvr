use chrono::{DateTime, Utc};
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::*;
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct TimelineBlock {
    pub recording_id: Uuid,
    pub camera_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub recording_type: String,
    pub has_audio: bool,
}

#[derive(Debug, Serialize)]
pub struct TimelineEvent {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub event_type: String,
    pub confidence: Option<f32>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TimelineResponse {
    pub camera_id: Uuid,
    pub recordings: Vec<TimelineBlock>,
    pub events: Vec<TimelineEvent>,
}

pub struct TimelineQueryService {
    recording_repo: Arc<dyn RecordingRepository>,
    event_repo: Arc<dyn DetectionEventRepository>,
}

impl TimelineQueryService {
    pub fn new(
        recording_repo: Arc<dyn RecordingRepository>,
        event_repo: Arc<dyn DetectionEventRepository>,
    ) -> Self {
        Self {
            recording_repo,
            event_repo,
        }
    }

    pub async fn get_timeline(
        &self,
        camera_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TimelineResponse, DomainError> {
        let recordings = self.recording_repo.find_by_camera(camera_id, start, end).await?;

        let events = self.event_repo.find_by_camera(camera_id, start, end).await?;

        Ok(TimelineResponse {
            camera_id,
            recordings: recordings
                .into_iter()
                .map(|r| TimelineBlock {
                    recording_id: r.id,
                    camera_id: r.camera_id,
                    start_time: r.start_time,
                    end_time: r.end_time,
                    recording_type: r.recording_type.to_string(),
                    has_audio: r.has_audio,
                })
                .collect(),
            events: events
                .into_iter()
                .map(|e| TimelineEvent {
                    id: e.id,
                    camera_id: e.camera_id,
                    event_type: e.event_type.to_string(),
                    confidence: e.confidence,
                    occurred_at: e.occurred_at,
                })
                .collect(),
        })
    }
}
