use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use open_nvr_domain::errors::DomainError;
use serde_json::json;

pub struct ApiError(DomainError);

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            DomainError::NotFound { .. } => (StatusCode::NOT_FOUND, self.0.to_string()),
            DomainError::Validation(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            DomainError::Duplicate(_) => (StatusCode::CONFLICT, self.0.to_string()),
            DomainError::Authentication(_) => (StatusCode::UNAUTHORIZED, self.0.to_string()),
            DomainError::Authorization(_) => (StatusCode::FORBIDDEN, self.0.to_string()),
            DomainError::CameraConnection(_) => (StatusCode::BAD_GATEWAY, self.0.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        tracing::error!(error = %self.0, status = %status, "API error");

        let body = json!({
            "error": {
                "code": status.as_u16(),
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
