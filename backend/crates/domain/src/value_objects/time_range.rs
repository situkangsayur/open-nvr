use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, DomainError> {
        if end <= start {
            return Err(DomainError::Validation(
                "Time range end must be after start".into(),
            ));
        }
        Ok(Self { start, end })
    }

    pub fn duration_secs(&self) -> i64 {
        (self.end - self.start).num_seconds()
    }

    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        time >= self.start && time <= self.end
    }

    pub fn overlaps(&self, other: &TimeRange) -> bool {
        self.start < other.end && other.start < self.end
    }
}
