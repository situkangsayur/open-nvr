use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/cameras/{camera_id}/zones",
            get(list_zones).post(create_zone),
        )
        .route(
            "/cameras/{camera_id}/zones/{zone_id}",
            get(get_zone).delete(delete_zone),
        )
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct CreateZoneRequest {
    pub name: String,
    pub polygon: Vec<Point>,
    pub detection_types: Option<Vec<String>>,
    pub sensitivity: Option<f32>,
}

impl CreateZoneRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 255 {
            return Err("Zone name must be 1-255 characters".into());
        }
        if self.polygon.len() < 3 {
            return Err("Polygon must have at least 3 points".into());
        }
        if self.polygon.len() > 50 {
            return Err("Polygon must have at most 50 points".into());
        }
        for point in &self.polygon {
            if point.x < 0.0 || point.x > 1.0 || point.y < 0.0 || point.y > 1.0 {
                return Err("Polygon points must be normalized (0.0-1.0)".into());
            }
        }
        if let Some(s) = self.sensitivity {
            if !(0.0..=1.0).contains(&s) {
                return Err("Sensitivity must be between 0.0 and 1.0".into());
            }
        }
        Ok(())
    }
}

async fn list_zones(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
) -> ApiResult<Json<Vec<DetectionZone>>> {
    let zones = state.zone_repo.find_by_camera(camera_id).await?;
    Ok(Json(zones))
}

async fn get_zone(
    State(state): State<AppState>,
    Path((_camera_id, zone_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<DetectionZone>> {
    let zone = state
        .zone_repo
        .find_by_id(zone_id)
        .await?
        .ok_or(DomainError::NotFound {
            entity_type: "detection_zone".into(),
            id: zone_id,
        })?;
    Ok(Json(zone))
}

async fn create_zone(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Json(req): Json<CreateZoneRequest>,
) -> ApiResult<Json<DetectionZone>> {
    req.validate()
        .map_err(|e| open_nvr_domain::errors::DomainError::Validation(e))?;

    let detection_types = req
        .detection_types
        .map(|types| {
            types
                .iter()
                .filter_map(|t| t.parse::<DetectionEventType>().ok())
                .collect()
        })
        .unwrap_or_else(|| vec![DetectionEventType::Motion]);

    let mut zone = DetectionZone::new(camera_id, req.name, req.polygon);
    zone.detection_types = detection_types;
    if let Some(s) = req.sensitivity {
        zone.sensitivity = s;
    }

    state.zone_repo.create(&zone).await?;
    Ok(Json(zone))
}

async fn delete_zone(
    State(state): State<AppState>,
    Path((_camera_id, zone_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<serde_json::Value>> {
    state.zone_repo.delete(zone_id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
