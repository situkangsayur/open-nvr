use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub name: String,
    pub camera_id: Option<Uuid>,
    pub retention_days: i32,
    pub max_storage_bytes: Option<i64>,
    pub recording_type: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RetentionPolicy {
    pub fn new_global(name: String, retention_days: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            camera_id: None,
            retention_days,
            max_storage_bytes: None,
            recording_type: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}
