use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/cameras/{id}/ptz", post(ptz_command))
        .route("/cameras/{id}/ptz/preset", post(goto_preset))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct PtzCommand {
    pub action: String,      // pan_left, pan_right, tilt_up, tilt_down, zoom_in, zoom_out, stop, home
    pub speed: Option<f32>,  // 0.0-1.0
}

impl PtzCommand {
    pub fn validate(&self) -> Result<(), String> {
        let valid_actions = ["pan_left", "pan_right", "tilt_up", "tilt_down",
                            "zoom_in", "zoom_out", "stop", "home"];
        if !valid_actions.contains(&self.action.as_str()) {
            return Err(format!("Invalid PTZ action: {}. Valid: {:?}", self.action, valid_actions));
        }
        if let Some(speed) = self.speed {
            if !(0.0..=1.0).contains(&speed) {
                return Err("Speed must be between 0.0 and 1.0".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct PresetCommand {
    pub preset_id: u32,
}

impl PresetCommand {
    pub fn validate(&self) -> Result<(), String> {
        if self.preset_id > 255 {
            return Err("Preset ID must be 0-255".into());
        }
        Ok(())
    }
}

async fn ptz_command(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Json(cmd): Json<PtzCommand>,
) -> ApiResult<Json<serde_json::Value>> {
    cmd.validate().map_err(|e| open_nvr_domain::errors::DomainError::Validation(e))?;

    // Verify camera exists and is PTZ capable
    let camera = state.camera_queries.get_camera(camera_id).await?;
    if !camera.ptz_capable {
        return Err(open_nvr_domain::errors::DomainError::Validation(
            "Camera is not PTZ capable".into()
        ).into());
    }

    // Log PTZ action to audit
    let audit = open_nvr_domain::entities::AuditLog::new(
        format!("ptz.{}", cmd.action),
        "camera".into(),
        Some(camera_id.to_string()),
    ).with_details(serde_json::json!({
        "action": cmd.action,
        "speed": cmd.speed,
    }));
    let _ = state.audit_repo.log(&audit).await;

    // TODO: Send actual PTZ command via ONVIF when protocol adapter supports it
    tracing::info!(camera_id = %camera_id, action = %cmd.action, "PTZ command");

    Ok(Json(serde_json::json!({
        "status": "ok",
        "camera_id": camera_id,
        "action": cmd.action,
        "speed": cmd.speed.unwrap_or(0.5),
    })))
}

async fn goto_preset(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Json(cmd): Json<PresetCommand>,
) -> ApiResult<Json<serde_json::Value>> {
    cmd.validate().map_err(|e| open_nvr_domain::errors::DomainError::Validation(e))?;

    let camera = state.camera_queries.get_camera(camera_id).await?;
    if !camera.ptz_capable {
        return Err(open_nvr_domain::errors::DomainError::Validation(
            "Camera is not PTZ capable".into()
        ).into());
    }

    let audit = open_nvr_domain::entities::AuditLog::new(
        "ptz.preset".into(),
        "camera".into(),
        Some(camera_id.to_string()),
    ).with_details(serde_json::json!({ "preset_id": cmd.preset_id }));
    let _ = state.audit_repo.log(&audit).await;

    tracing::info!(camera_id = %camera_id, preset = cmd.preset_id, "PTZ goto preset");

    Ok(Json(serde_json::json!({
        "status": "ok",
        "camera_id": camera_id,
        "preset_id": cmd.preset_id,
    })))
}
