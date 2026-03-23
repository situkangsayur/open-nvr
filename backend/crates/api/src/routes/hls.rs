use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/hls/{camera_id}/{filename}", get(serve_hls))
        .route("/hls/{camera_id}/start", axum::routing::post(start_hls))
        .with_state(state)
}

/// Serve HLS playlist or segment files
async fn serve_hls(
    State(state): State<AppState>,
    Path((camera_id, filename)): Path<(Uuid, String)>,
) -> Response {
    // Validate filename (security: prevent path traversal)
    if filename.contains("..") || filename.contains('/') {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let hls_mgr = match &state.hls_manager {
        Some(m) => m,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "HLS not configured").into_response(),
    };

    let file_path = hls_mgr.camera_dir(&camera_id).join(&filename);

    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            let content_type = if filename.ends_with(".m3u8") {
                "application/vnd.apple.mpegurl"
            } else if filename.ends_with(".ts") {
                "video/mp2t"
            } else {
                "application/octet-stream"
            };

            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-cache, no-store"),
                    (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
                ],
                data,
            )
                .into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Start HLS stream for a camera
async fn start_hls(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
) -> Response {
    let camera = match state.camera_queries.get_camera(camera_id).await {
        Ok(c) => c,
        Err(_) => return (StatusCode::NOT_FOUND, "Camera not found").into_response(),
    };

    let hls_mgr = match &state.hls_manager {
        Some(m) => m,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "HLS not configured").into_response(),
    };

    match hls_mgr.start(&camera).await {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "status": "started",
                "playlist": format!("/api/hls/{}/stream.m3u8", camera_id),
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "error": e,
            })),
        )
            .into_response(),
    }
}
