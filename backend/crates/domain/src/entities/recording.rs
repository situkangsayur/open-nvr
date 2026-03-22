use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingType {
    Continuous,
    Motion,
}

impl std::fmt::Display for RecordingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continuous => write!(f, "continuous"),
            Self::Motion => write!(f, "motion"),
        }
    }
}

impl std::str::FromStr for RecordingType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "continuous" => Ok(Self::Continuous),
            "motion" => Ok(Self::Motion),
            _ => Err(format!("Unknown recording type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordingStatus {
    Recording,
    Completed,
    Error,
}

impl std::fmt::Display for RecordingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recording => write!(f, "recording"),
            Self::Completed => write!(f, "completed"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for RecordingStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "recording" => Ok(Self::Recording),
            "completed" => Ok(Self::Completed),
            "error" => Ok(Self::Error),
            _ => Err(format!("Unknown recording status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub recording_type: RecordingType,
    pub has_audio: bool,
    pub total_size: i64,
    pub status: RecordingStatus,
    pub created_at: DateTime<Utc>,
}

impl Recording {
    pub fn new_continuous(camera_id: Uuid, has_audio: bool) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            camera_id,
            start_time: now,
            end_time: None,
            recording_type: RecordingType::Continuous,
            has_audio,
            total_size: 0,
            status: RecordingStatus::Recording,
            created_at: now,
        }
    }

    pub fn complete(&mut self) {
        self.end_time = Some(Utc::now());
        self.status = RecordingStatus::Completed;
    }

    pub fn duration_secs(&self) -> Option<i64> {
        self.end_time.map(|end| (end - self.start_time).num_seconds())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSegment {
    pub id: Uuid,
    pub recording_id: Uuid,
    pub sequence_number: i32,
    pub storage_key: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

impl RecordingSegment {
    pub fn new(recording_id: Uuid, sequence_number: i32, storage_key: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            recording_id,
            sequence_number,
            storage_key,
            start_time: now,
            end_time: None,
            duration_ms: None,
            size_bytes: 0,
            created_at: now,
        }
    }

    pub fn storage_key_for(camera_id: Uuid, time: DateTime<Utc>, seq: i32) -> String {
        format!(
            "recordings/{}/{}/{}/{}/{}_{}.mp4",
            camera_id,
            time.format("%Y"),
            time.format("%m"),
            time.format("%d"),
            time.timestamp(),
            seq
        )
    }
}
