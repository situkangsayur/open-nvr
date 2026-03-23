use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::middleware::Claims;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/user/change-password", post(change_password))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

impl ChangePasswordRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.new_password.len() < 8 {
            return Err("New password must be at least 8 characters".into());
        }
        if self.current_password == self.new_password {
            return Err("New password must be different from current password".into());
        }
        if self.new_password.len() > 255 {
            return Err("Password too long".into());
        }
        Ok(())
    }
}

async fn change_password(
    claims: Option<axum::Extension<Claims>>,
    Json(req): Json<ChangePasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    req.validate().map_err(|e| open_nvr_domain::errors::DomainError::Validation(e))?;

    let user_claims = claims.as_ref()
        .ok_or(open_nvr_domain::errors::DomainError::Authentication("Not authenticated".into()))?;

    let username = user_claims.preferred_username.clone()
        .or(Some(user_claims.sub.clone()))
        .unwrap_or_default();

    let keycloak_url = std::env::var("KEYCLOAK_URL").unwrap_or("http://localhost:8190".into());
    let realm = std::env::var("KEYCLOAK_REALM").unwrap_or("opennvr".into());
    let client = reqwest::Client::new();

    // Step 1: Verify current password
    let token_url = format!("{}/realms/{}/protocol/openid-connect/token", keycloak_url, realm);
    let verify_resp = client.post(&token_url)
        .form(&[
            ("grant_type", "password"),
            ("client_id", "opennvr-frontend"),
            ("username", &username),
            ("password", &req.current_password),
            ("scope", "openid"),
        ])
        .send()
        .await
        .map_err(|e| open_nvr_domain::errors::DomainError::Internal(e.to_string()))?;

    if !verify_resp.status().is_success() {
        return Err(open_nvr_domain::errors::DomainError::Authentication("Current password is incorrect".into()).into());
    }

    // Step 2: Get admin token
    let admin_token_resp = client.post(&format!("{}/realms/master/protocol/openid-connect/token", keycloak_url))
        .form(&[
            ("grant_type", "password"),
            ("client_id", "admin-cli"),
            ("username", "admin"),
            ("password", &std::env::var("KEYCLOAK_ADMIN_PASSWORD").unwrap_or("admin".into())),
        ])
        .send()
        .await
        .map_err(|e| open_nvr_domain::errors::DomainError::Internal(e.to_string()))?;

    let admin_token: serde_json::Value = admin_token_resp.json().await
        .map_err(|e| open_nvr_domain::errors::DomainError::Internal(e.to_string()))?;

    let access_token = admin_token["access_token"].as_str()
        .ok_or(open_nvr_domain::errors::DomainError::Internal("Failed to get admin token".into()))?;

    // Step 3: Find user by username
    let users_url = format!("{}/admin/realms/{}/users?username={}", keycloak_url, realm, username);
    let users_resp = client.get(&users_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| open_nvr_domain::errors::DomainError::Internal(e.to_string()))?;

    let users: Vec<serde_json::Value> = users_resp.json().await
        .map_err(|e| open_nvr_domain::errors::DomainError::Internal(e.to_string()))?;

    let user_id = users.first()
        .and_then(|u| u["id"].as_str())
        .ok_or(open_nvr_domain::errors::DomainError::NotFound { entity_type: "user".into(), id: uuid::Uuid::nil() })?;

    // Step 4: Reset password
    let reset_url = format!("{}/admin/realms/{}/users/{}/reset-password", keycloak_url, realm, user_id);
    let reset_resp = client.put(&reset_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&serde_json::json!({
            "type": "password",
            "value": req.new_password,
            "temporary": false
        }))
        .send()
        .await
        .map_err(|e| open_nvr_domain::errors::DomainError::Internal(e.to_string()))?;

    if reset_resp.status().is_success() || reset_resp.status().as_u16() == 204 {
        // Audit log
        tracing::info!(username = %username, "Password changed");
        Ok(Json(serde_json::json!({"status": "ok", "message": "Password changed successfully"})))
    } else {
        let err_body = reset_resp.text().await.unwrap_or_default();
        Err(open_nvr_domain::errors::DomainError::Internal(format!("Failed to change password: {}", err_body)).into())
    }
}
