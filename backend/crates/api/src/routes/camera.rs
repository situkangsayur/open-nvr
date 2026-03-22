use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use open_nvr_application::dto::*;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::Claims;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/cameras", get(list_cameras).post(create_camera))
        .route("/cameras/{id}", get(get_camera).put(update_camera).delete(delete_camera))
        .with_state(state)
}

async fn list_cameras(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<CameraResponse>>> {
    let cameras = state.camera_queries.list_cameras().await?;
    let response: Vec<CameraResponse> = cameras.into_iter().map(|c| c.into()).collect();
    Ok(Json(response))
}

async fn get_camera(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CameraResponse>> {
    let camera = state.camera_queries.get_camera(id).await?;
    Ok(Json(camera.into()))
}

async fn create_camera(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
    Json(req): Json<CreateCameraRequest>,
) -> ApiResult<Json<CameraResponse>> {
    let (user_id, user_email) = extract_user_info(&claims);
    let camera = state.camera_commands.create_camera(req, &user_id, user_email.as_deref()).await?;
    Ok(Json(camera.into()))
}

async fn update_camera(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCameraRequest>,
) -> ApiResult<Json<CameraResponse>> {
    let (user_id, user_email) = extract_user_info(&claims);
    let camera = state.camera_commands.update_camera(id, req, &user_id, user_email.as_deref()).await?;
    Ok(Json(camera.into()))
}

async fn delete_camera(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let (user_id, user_email) = extract_user_info(&claims);
    state.camera_commands.delete_camera(id, &user_id, user_email.as_deref()).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

fn extract_user_info(claims: &Option<axum::Extension<Claims>>) -> (String, Option<String>) {
    match claims {
        Some(c) => (c.sub.clone(), c.email.clone()),
        None => ("anonymous".to_string(), None),
    }
}
