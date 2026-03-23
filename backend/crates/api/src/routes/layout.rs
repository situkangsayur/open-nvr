use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::Claims;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/layouts", get(list_layouts).post(create_layout))
        .route("/layouts/{id}", get(get_layout).put(update_layout).delete(delete_layout))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct CreateLayoutRequest {
    pub name: String,
    pub layout_type: Option<String>,
    pub camera_positions: Vec<CameraPosition>,
    pub is_default: Option<bool>,
}

impl CreateLayoutRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 255 {
            return Err("Layout name must be 1-255 characters".into());
        }
        if self.camera_positions.len() > 64 {
            return Err("Maximum 64 camera positions per layout".into());
        }
        if let Some(ref lt) = self.layout_type {
            let valid = ["grid", "single", "l_shape", "custom"];
            if !valid.contains(&lt.as_str()) {
                return Err(format!("Invalid layout type: {}. Valid: {:?}", lt, valid));
            }
        }
        Ok(())
    }
}

async fn list_layouts(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
) -> ApiResult<Json<Vec<GridLayout>>> {
    let user_id = claims.as_ref().map(|c| c.sub.as_str()).unwrap_or("anonymous");
    let layouts = state.layout_repo.find_by_user(user_id).await?;
    Ok(Json(layouts))
}

async fn get_layout(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<GridLayout>> {
    let layout = state.layout_repo.find_by_id(id).await?
        .ok_or(DomainError::NotFound { entity_type: "grid_layout".into(), id })?;
    Ok(Json(layout))
}

async fn create_layout(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
    Json(req): Json<CreateLayoutRequest>,
) -> ApiResult<Json<GridLayout>> {
    req.validate().map_err(|e| DomainError::Validation(e))?;

    let user_id = claims.as_ref().map(|c| c.sub.clone()).unwrap_or("anonymous".into());
    let layout_type = req.layout_type
        .and_then(|lt| lt.parse::<LayoutType>().ok())
        .unwrap_or(LayoutType::Grid);

    let mut layout = GridLayout::new(user_id, req.name, layout_type);
    layout.camera_positions = req.camera_positions;
    layout.is_default = req.is_default.unwrap_or(false);

    state.layout_repo.create(&layout).await?;
    Ok(Json(layout))
}

async fn update_layout(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateLayoutRequest>,
) -> ApiResult<Json<GridLayout>> {
    req.validate().map_err(|e| DomainError::Validation(e))?;

    let mut layout = state.layout_repo.find_by_id(id).await?
        .ok_or(DomainError::NotFound { entity_type: "grid_layout".into(), id })?;

    layout.name = req.name;
    if let Some(lt) = req.layout_type {
        layout.layout_type = lt.parse().unwrap_or(LayoutType::Grid);
    }
    layout.camera_positions = req.camera_positions;
    layout.is_default = req.is_default.unwrap_or(layout.is_default);
    layout.updated_at = chrono::Utc::now();

    state.layout_repo.update(&layout).await?;
    Ok(Json(layout))
}

async fn delete_layout(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.layout_repo.delete(id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
