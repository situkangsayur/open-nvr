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

    // Build ONVIF SOAP request for PTZ control
    let camera_ip = extract_camera_ip(&camera.stream_url);
    let onvif_url = format!("http://{}:8899/onvif/PTZ", camera_ip);

    let soap_body = match cmd.action.as_str() {
        "pan_left" => format!(r#"<tptz:ContinuousMove><tptz:ProfileToken>stream0_0</tptz:ProfileToken><tptz:Velocity><tt:PanTilt x="-{}" y="0"/></tptz:Velocity></tptz:ContinuousMove>"#, cmd.speed.unwrap_or(0.5)),
        "pan_right" => format!(r#"<tptz:ContinuousMove><tptz:ProfileToken>stream0_0</tptz:ProfileToken><tptz:Velocity><tt:PanTilt x="{}" y="0"/></tptz:Velocity></tptz:ContinuousMove>"#, cmd.speed.unwrap_or(0.5)),
        "tilt_up" => format!(r#"<tptz:ContinuousMove><tptz:ProfileToken>stream0_0</tptz:ProfileToken><tptz:Velocity><tt:PanTilt x="0" y="{}"/></tptz:Velocity></tptz:ContinuousMove>"#, cmd.speed.unwrap_or(0.5)),
        "tilt_down" => format!(r#"<tptz:ContinuousMove><tptz:ProfileToken>stream0_0</tptz:ProfileToken><tptz:Velocity><tt:PanTilt x="0" y="-{}"/></tptz:Velocity></tptz:ContinuousMove>"#, cmd.speed.unwrap_or(0.5)),
        "zoom_in" => format!(r#"<tptz:ContinuousMove><tptz:ProfileToken>stream0_0</tptz:ProfileToken><tptz:Velocity><tt:Zoom x="{}"/></tptz:Velocity></tptz:ContinuousMove>"#, cmd.speed.unwrap_or(0.5)),
        "zoom_out" => format!(r#"<tptz:ContinuousMove><tptz:ProfileToken>stream0_0</tptz:ProfileToken><tptz:Velocity><tt:Zoom x="-{}"/></tptz:Velocity></tptz:ContinuousMove>"#, cmd.speed.unwrap_or(0.5)),
        "stop" => r#"<tptz:Stop><tptz:ProfileToken>stream0_0</tptz:ProfileToken><tptz:PanTilt>true</tptz:PanTilt><tptz:Zoom>true</tptz:Zoom></tptz:Stop>"#.to_string(),
        "home" => r#"<tptz:GotoHomePosition><tptz:ProfileToken>stream0_0</tptz:ProfileToken></tptz:GotoHomePosition>"#.to_string(),
        _ => return Err(open_nvr_domain::errors::DomainError::Validation("Invalid PTZ action".into()).into()),
    };

    let soap_envelope = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl" xmlns:tt="http://www.onvif.org/ver10/schema">
  <s:Body>{}</s:Body>
</s:Envelope>"#, soap_body);

    let client = reqwest::Client::new();
    let resp = client.post(&onvif_url)
        .header("Content-Type", "application/soap+xml")
        .body(soap_envelope)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    let ptz_success = match &resp {
        Ok(r) => r.status().is_success(),
        Err(e) => {
            tracing::warn!(camera_id = %camera_id, error = %e, "PTZ ONVIF request failed");
            false
        }
    };

    tracing::info!(camera_id = %camera_id, action = %cmd.action, success = ptz_success, "PTZ command sent via ONVIF");

    if ptz_success {
        Ok(Json(serde_json::json!({
            "status": "ok",
            "camera_id": camera_id,
            "action": cmd.action,
            "speed": cmd.speed.unwrap_or(0.5),
        })))
    } else {
        Ok(Json(serde_json::json!({
            "status": "error",
            "camera_id": camera_id,
            "action": cmd.action,
            "message": "PTZ command failed - camera may be unreachable",
        })))
    }
}

/// Extract camera IP from RTSP URL like `rtsp://user:pass@192.168.1.10:554/path`
fn extract_camera_ip(stream_url: &str) -> String {
    stream_url
        .split("://").nth(1)
        .and_then(|s| s.split('@').last())
        .and_then(|s| s.split(':').next())
        .and_then(|s| s.split('/').next())
        .unwrap_or("0.0.0.0")
        .to_string()
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
