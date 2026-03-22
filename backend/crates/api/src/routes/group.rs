use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use open_nvr_application::dto::*;
use open_nvr_domain::ports::CameraGroupRepository;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::Claims;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/{id}", get(get_group).delete(delete_group))
        .with_state(state)
}

async fn list_groups(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<GroupResponse>>> {
    use open_nvr_infrastructure::persistence::PgCameraGroupRepository;
    // Use the group repo directly for queries
    let repo = PgCameraGroupRepository::new(state.db_pool.clone());
    let groups = repo.find_all().await?;
    let response: Vec<GroupResponse> = groups.into_iter().map(|g| g.into()).collect();
    Ok(Json(response))
}

async fn get_group(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<GroupResponse>> {
    use open_nvr_infrastructure::persistence::PgCameraGroupRepository;
    let repo = PgCameraGroupRepository::new(state.db_pool.clone());
    let group = repo.find_by_id(id).await?
        .ok_or(open_nvr_domain::errors::DomainError::NotFound {
            entity_type: "camera_group".into(),
            id,
        })?;
    Ok(Json(group.into()))
}

async fn create_group(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
    Json(req): Json<CreateGroupRequest>,
) -> ApiResult<Json<GroupResponse>> {
    let user_id = claims.as_ref().map(|c| c.sub.clone()).unwrap_or("anonymous".into());
    let group = state.group_commands.create_group(req, &user_id).await?;
    Ok(Json(group.into()))
}

async fn delete_group(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_id = claims.as_ref().map(|c| c.sub.clone()).unwrap_or("anonymous".into());
    state.group_commands.delete_group(id, &user_id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
