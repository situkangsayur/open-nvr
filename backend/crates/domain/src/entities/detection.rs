use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DetectionEventType {
    Motion,
    Human,
    Animal,
    Vehicle,
    Unknown,
}

impl std::fmt::Display for DetectionEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Motion => write!(f, "motion"),
            Self::Human => write!(f, "human"),
            Self::Animal => write!(f, "animal"),
            Self::Vehicle => write!(f, "vehicle"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for DetectionEventType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "motion" => Ok(Self::Motion),
            "human" => Ok(Self::Human),
            "animal" => Ok(Self::Animal),
            "vehicle" => Ok(Self::Vehicle),
            "unknown" => Ok(Self::Unknown),
            _ => Err(format!("Unknown event type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionEvent {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub zone_id: Option<Uuid>,
    pub event_type: DetectionEventType,
    pub confidence: Option<f32>,
    pub bounding_box: Option<BoundingBox>,
    pub thumbnail_key: Option<String>,
    pub metadata: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionZone {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub name: String,
    pub polygon: Vec<Point>,
    pub detection_types: Vec<DetectionEventType>,
    pub sensitivity: f32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DetectionZone {
    pub fn new(camera_id: Uuid, name: String, polygon: Vec<Point>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            camera_id,
            name,
            polygon,
            detection_types: vec![DetectionEventType::Motion],
            sensitivity: 0.5,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if a point (normalized 0.0-1.0) is inside this zone's polygon
    /// Using ray casting algorithm
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        let n = self.polygon.len();
        if n < 3 {
            return false;
        }
        let mut inside = false;
        let mut j = n - 1;
        for i in 0..n {
            let pi = &self.polygon[i];
            let pj = &self.polygon[j];
            if ((pi.y > py) != (pj.y > py))
                && (px < (pj.x - pi.x) * (py - pi.y) / (pj.y - pi.y) + pi.x)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }
}
