use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// This realm's tokens are issued without `sub`, so it is filled in from
    /// `preferred_username` after decoding — see `auth_middleware`.
    #[serde(default)]
    pub sub: String,
    pub iss: String,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

impl Claims {
    pub fn has_role(&self, role: &str) -> bool {
        self.realm_access
            .as_ref()
            .map(|ra| ra.roles.iter().any(|r| r == role))
            .unwrap_or(false)
    }

    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }
}

#[derive(Clone)]
pub struct KeycloakConfig {
    pub realm_url: String,
    /// `/realms/<realm>` — the tail every issuer for this realm ends with.
    pub realm_path: String,
    pub decoding_key: Arc<RwLock<Option<DecodingKey>>>,
}

impl KeycloakConfig {
    pub fn new(keycloak_url: &str, realm: &str) -> Self {
        Self {
            realm_url: format!("{}/realms/{}", keycloak_url, realm),
            realm_path: format!("/realms/{}", realm),
            decoding_key: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn fetch_public_key(&self) -> Result<(), String> {
        let url = format!("{}/protocol/openid-connect/certs", self.realm_url);
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

        let jwks: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse JWKS: {}", e))?;

        // Get the first RSA key from JWKS
        if let Some(keys) = jwks.get("keys").and_then(|k| k.as_array()) {
            for key in keys {
                if key.get("kty").and_then(|v| v.as_str()) == Some("RSA") {
                    if let (Some(n), Some(e)) = (
                        key.get("n").and_then(|v| v.as_str()),
                        key.get("e").and_then(|v| v.as_str()),
                    ) {
                        let decoding_key = DecodingKey::from_rsa_components(n, e)
                            .map_err(|e| format!("Failed to create decoding key: {}", e))?;
                        *self.decoding_key.write().await = Some(decoding_key);
                        return Ok(());
                    }
                }
            }
        }

        Err("No RSA key found in JWKS".into())
    }
}

/// Pull `access_token` out of a raw query string. JWTs are base64url, so the
/// value needs no percent-decoding.
fn token_from_query(query: Option<&str>) -> Option<String> {
    query?
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == "access_token")
        .map(|(_, value)| value.to_string())
}

/// Axum middleware that validates JWT tokens from Keycloak
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Extract KeycloakConfig from extensions
    let keycloak_config = request
        .extensions()
        .get::<KeycloakConfig>()
        .cloned();

    let keycloak_config = match keycloak_config {
        Some(config) => config,
        None => {
            warn!("KeycloakConfig not found in request extensions");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Auth not configured"})),
            )
                .into_response();
        }
    };

    // Authorization header first, then `?access_token=`. The query form exists
    // because `<video src>` and `<img src>` cannot carry headers, and HLS
    // playlists, segments and snapshots all live under the protected /api tree.
    let token = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_owned)
        .or_else(|| token_from_query(request.uri().query()));

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Missing or invalid Authorization header"})),
            )
                .into_response();
        }
    };

    // Validate token
    let decoding_key = keycloak_config.decoding_key.read().await;
    let decoding_key = match decoding_key.as_ref() {
        Some(key) => key.clone(),
        None => {
            drop(decoding_key);
            // Try to fetch key
            if let Err(e) = keycloak_config.fetch_public_key().await {
                warn!("Failed to fetch Keycloak public key: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Auth service unavailable"})),
                )
                    .into_response();
            }
            let key = keycloak_config.decoding_key.read().await;
            match key.as_ref() {
                Some(k) => k.clone(),
                None => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to obtain auth key"})),
                    )
                        .into_response();
                }
            }
        }
    };

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.validate_aud = false;

    match decode::<Claims>(&token, &decoding_key, &validation) {
        Ok(token_data) => {
            // The same Keycloak answers on the LAN address and on the WireGuard
            // address, so `iss` differs by host between clients. Pin the realm
            // path instead — the RSA signature is what ties the token to us.
            if !token_data.claims.iss.ends_with(&keycloak_config.realm_path) {
                warn!(iss = %token_data.claims.iss, "Token issued by an unexpected realm");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Invalid or expired token"})),
                )
                    .into_response();
            }
            let mut claims = token_data.claims;
            if claims.sub.is_empty() {
                // Audit rows still need a name for whoever did the thing.
                claims.sub = claims
                    .preferred_username
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string());
            }

            let mut request = request;
            request.extensions_mut().insert(claims);
            next.run(request).await
        }
        Err(e) => {
            warn!("JWT validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid or expired token"})),
            )
                .into_response()
        }
    }
}

/// Middleware to require a specific role
pub async fn require_role(
    role: &str,
    request: &Request,
) -> Result<(), Response> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Not authenticated"})),
            )
                .into_response()
        })?;

    if !claims.has_role(role) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": format!("Required role: {}", role)})),
        )
            .into_response());
    }

    Ok(())
}
