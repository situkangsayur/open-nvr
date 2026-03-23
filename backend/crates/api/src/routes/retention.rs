use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use open_nvr_domain::entities::RetentionPolicy;
use open_nvr_domain::errors::DomainError;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/settings/retention", get(list_policies).post(create_policy))
        .route("/settings/retention/{id}", get(get_policy).delete(delete_policy))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct CreateRetentionRequest {
    pub name: String,
    pub camera_id: Option<Uuid>,
    pub retention_days: i32,
    pub max_storage_bytes: Option<i64>,
    pub recording_type: Option<String>,
}

impl CreateRetentionRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 255 {
            return Err("Policy name must be 1-255 characters".into());
        }
        if self.retention_days < 1 || self.retention_days > 3650 {
            return Err("Retention days must be 1-3650".into());
        }
        if let Some(bytes) = self.max_storage_bytes {
            if bytes < 0 {
                return Err("Max storage bytes must be non-negative".into());
            }
        }
        if let Some(ref rt) = self.recording_type {
            let valid = ["all", "continuous", "motion"];
            if !valid.contains(&rt.as_str()) {
                return Err(format!("Invalid recording type: {}. Valid: {:?}", rt, valid));
            }
        }
        Ok(())
    }
}

async fn list_policies(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<RetentionPolicy>>> {
    let policies = state.retention_repo.find_all().await?;
    Ok(Json(policies))
}

async fn get_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RetentionPolicy>> {
    let policies = state.retention_repo.find_all().await?;
    let policy = policies.into_iter().find(|p| p.id == id)
        .ok_or(DomainError::NotFound { entity_type: "retention_policy".into(), id })?;
    Ok(Json(policy))
}

async fn create_policy(
    State(state): State<AppState>,
    Json(req): Json<CreateRetentionRequest>,
) -> ApiResult<Json<RetentionPolicy>> {
    req.validate().map_err(|e| DomainError::Validation(e))?;

    let mut policy = RetentionPolicy::new_global(req.name, req.retention_days);
    policy.camera_id = req.camera_id;
    policy.max_storage_bytes = req.max_storage_bytes;
    policy.recording_type = req.recording_type;

    state.retention_repo.create(&policy).await?;
    Ok(Json(policy))
}

async fn delete_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    state.retention_repo.delete(id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
