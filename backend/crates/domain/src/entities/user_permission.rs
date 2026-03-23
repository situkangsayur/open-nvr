use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCameraAccess {
    pub id: Uuid,
    pub user_id: String,
    pub camera_id: Uuid,
    pub can_view: bool,
    pub can_ptz: bool,
    pub can_playback: bool,
    pub can_export: bool,
    pub granted_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl UserCameraAccess {
    pub fn new(user_id: String, camera_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            camera_id,
            can_view: true,
            can_ptz: false,
            can_playback: true,
            can_export: false,
            granted_by: None,
            created_at: Utc::now(),
        }
    }

    pub fn full_access(user_id: String, camera_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            camera_id,
            can_view: true,
            can_ptz: true,
            can_playback: true,
            can_export: true,
            granted_by: None,
            created_at: Utc::now(),
        }
    }
}

/// Role-based permission levels
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    Operator,
    Viewer,
}

impl UserRole {
    pub fn from_roles(roles: &[String]) -> Self {
        if roles.iter().any(|r| r == "admin") {
            Self::Admin
        } else if roles.iter().any(|r| r == "operator") {
            Self::Operator
        } else {
            Self::Viewer
        }
    }

    pub fn can_manage_cameras(&self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_manage_settings(&self) -> bool {
        matches!(self, Self::Admin)
    }

    pub fn can_ptz(&self) -> bool {
        matches!(self, Self::Admin | Self::Operator)
    }

    pub fn can_export(&self) -> bool {
        matches!(self, Self::Admin | Self::Operator)
    }

    pub fn can_view_all_cameras(&self) -> bool {
        matches!(self, Self::Admin | Self::Operator)
    }
}
