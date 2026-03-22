use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LayoutType {
    Grid,
    Single,
    LShape,
    Custom,
}

impl std::fmt::Display for LayoutType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Grid => write!(f, "grid"),
            Self::Single => write!(f, "single"),
            Self::LShape => write!(f, "l_shape"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for LayoutType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grid" => Ok(Self::Grid),
            "single" => Ok(Self::Single),
            "l_shape" => Ok(Self::LShape),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Unknown layout type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPosition {
    pub camera_id: Uuid,
    pub row: u32,
    pub col: u32,
    pub row_span: u32,
    pub col_span: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridLayout {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub layout_type: LayoutType,
    pub camera_positions: Vec<CameraPosition>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GridLayout {
    pub fn new(user_id: String, name: String, layout_type: LayoutType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            name,
            layout_type,
            camera_positions: Vec::new(),
            is_default: false,
            created_at: now,
            updated_at: now,
        }
    }
}
