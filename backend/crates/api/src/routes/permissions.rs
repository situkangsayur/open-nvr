use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::UserPermissionRepository;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::Claims;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/permissions/users/{user_id}", get(get_user_permissions))
        .route(
            "/permissions/cameras/{camera_id}",
            get(get_camera_permissions),
        )
        .route("/permissions/grant", post(grant_permission))
        .route(
            "/permissions/revoke/{user_id}/{camera_id}",
            delete(revoke_permission),
        )
        .route("/permissions/my-cameras", get(my_cameras))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct GrantRequest {
    pub user_id: String,
    pub camera_id: Uuid,
    pub can_view: Option<bool>,
    pub can_ptz: Option<bool>,
    pub can_playback: Option<bool>,
    pub can_export: Option<bool>,
}

impl GrantRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.user_id.trim().is_empty() || self.user_id.len() > 255 {
            return Err("User ID must be 1-255 characters".into());
        }
        Ok(())
    }
}

async fn get_user_permissions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    claims: Option<axum::Extension<Claims>>,
) -> ApiResult<Json<Vec<UserCameraAccess>>> {
    // Only admin can view other users' permissions
    if let Some(ref c) = claims {
        if !c.is_admin() && c.sub != user_id {
            return Err(DomainError::Authorization(
                "Only admins can view other users' permissions".into(),
            )
            .into());
        }
    }
    let perms = state.permission_repo.find_by_user(&user_id).await?;
    Ok(Json(perms))
}

async fn get_camera_permissions(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    claims: Option<axum::Extension<Claims>>,
) -> ApiResult<Json<Vec<UserCameraAccess>>> {
    // Only admin can view camera permissions
    if let Some(ref c) = claims {
        if !c.is_admin() {
            return Err(DomainError::Authorization("Admin only".into()).into());
        }
    }
    let perms = state.permission_repo.find_by_camera(camera_id).await?;
    Ok(Json(perms))
}

async fn grant_permission(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
    Json(req): Json<GrantRequest>,
) -> ApiResult<Json<UserCameraAccess>> {
    req.validate()
        .map_err(|e| DomainError::Validation(e))?;

    // Only admin can grant permissions
    if let Some(ref c) = claims {
        if !c.is_admin() {
            return Err(
                DomainError::Authorization("Only admins can grant permissions".into()).into(),
            );
        }
    }

    let granted_by = claims
        .as_ref()
        .map(|c| c.sub.clone())
        .unwrap_or_else(|| "system".into());

    let mut access = UserCameraAccess::new(req.user_id, req.camera_id);
    access.can_view = req.can_view.unwrap_or(true);
    access.can_ptz = req.can_ptz.unwrap_or(false);
    access.can_playback = req.can_playback.unwrap_or(true);
    access.can_export = req.can_export.unwrap_or(false);
    access.granted_by = Some(granted_by);

    state.permission_repo.grant(&access).await?;

    // Audit
    let audit = AuditLog::new(
        "permission.grant".into(),
        "user_camera_access".into(),
        Some(access.camera_id.to_string()),
    )
    .with_details(serde_json::json!({
        "user_id": access.user_id,
        "camera_id": access.camera_id,
        "can_view": access.can_view,
        "can_ptz": access.can_ptz,
    }));
    let _ = state.audit_repo.log(&audit).await;

    Ok(Json(access))
}

async fn revoke_permission(
    State(state): State<AppState>,
    Path((user_id, camera_id)): Path<(String, Uuid)>,
    claims: Option<axum::Extension<Claims>>,
) -> ApiResult<Json<serde_json::Value>> {
    if let Some(ref c) = claims {
        if !c.is_admin() {
            return Err(
                DomainError::Authorization("Only admins can revoke permissions".into()).into(),
            );
        }
    }

    state.permission_repo.revoke(&user_id, camera_id).await?;

    let audit = AuditLog::new(
        "permission.revoke".into(),
        "user_camera_access".into(),
        Some(camera_id.to_string()),
    )
    .with_details(serde_json::json!({ "user_id": user_id }));
    let _ = state.audit_repo.log(&audit).await;

    Ok(Json(serde_json::json!({"revoked": true})))
}

/// Get cameras accessible by the current user
async fn my_cameras(
    State(state): State<AppState>,
    claims: Option<axum::Extension<Claims>>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let all_cameras = state.camera_queries.list_cameras().await?;

    let (user_id, roles) = match &claims {
        Some(c) => (
            c.sub.clone(),
            c.realm_access
                .as_ref()
                .map(|ra| ra.roles.clone())
                .unwrap_or_default(),
        ),
        None => ("anonymous".into(), vec![]),
    };

    let role = UserRole::from_roles(&roles);

    // Admin and operator see all cameras
    if role.can_view_all_cameras() {
        let result: Vec<serde_json::Value> = all_cameras
            .iter()
            .map(|cam| {
                serde_json::json!({
                    "camera": cam,
                    "permissions": {
                        "can_view": true,
                        "can_ptz": role.can_ptz(),
                        "can_playback": true,
                        "can_export": role.can_export(),
                    }
                })
            })
            .collect();
        return Ok(Json(result));
    }

    // Viewer: only show cameras with explicit access
    let user_perms = state.permission_repo.find_by_user(&user_id).await?;
    let allowed_camera_ids: std::collections::HashSet<Uuid> =
        user_perms.iter().map(|p| p.camera_id).collect();

    let result: Vec<serde_json::Value> = all_cameras
        .iter()
        .filter(|cam| allowed_camera_ids.contains(&cam.id))
        .map(|cam| {
            let perm = user_perms.iter().find(|p| p.camera_id == cam.id);
            serde_json::json!({
                "camera": cam,
                "permissions": {
                    "can_view": perm.map(|p| p.can_view).unwrap_or(false),
                    "can_ptz": perm.map(|p| p.can_ptz).unwrap_or(false),
                    "can_playback": perm.map(|p| p.can_playback).unwrap_or(false),
                    "can_export": perm.map(|p| p.can_export).unwrap_or(false),
                }
            })
        })
        .collect();

    Ok(Json(result))
}
