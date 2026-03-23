use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use open_nvr_infrastructure::protocols::create_stream_ingester;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/cameras/{id}/test", post(test_connection))
        .route("/cameras/{id}/start", post(start_camera))
        .route("/cameras/{id}/stop", post(stop_camera))
        .route("/cameras/{id}/snapshot", axum::routing::get(get_snapshot))
        .route("/cameras/test-url", post(test_url))
        .with_state(state)
}

/// Test connection to a camera by its ID
async fn test_connection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let camera = state.camera_queries.get_camera(id).await?;

    let mut ingester = create_stream_ingester(
        &camera.protocol_type.to_string(),
        &camera.stream_url,
        None,
        None,
    )
    .map_err(|e| open_nvr_domain::errors::DomainError::CameraConnection(e.to_string()))?;

    match tokio::time::timeout(std::time::Duration::from_secs(10), ingester.connect()).await {
        Ok(Ok(())) => {
            let info = ingester.stream_info();
            let _ = ingester.disconnect().await;
            Ok(Json(serde_json::json!({
                "status": "ok",
                "camera_id": id,
                "stream_info": info.map(|i| serde_json::json!({
                    "video_codec": i.video_codec,
                    "audio_codec": i.audio_codec,
                    "width": i.width,
                    "height": i.height,
                    "fps": i.fps,
                })),
            })))
        }
        Ok(Err(e)) => Ok(Json(serde_json::json!({
            "status": "error",
            "camera_id": id,
            "error": e.to_string(),
        }))),
        Err(_) => Ok(Json(serde_json::json!({
            "status": "timeout",
            "camera_id": id,
            "error": "Connection timed out after 10 seconds",
        }))),
    }
}

#[derive(Debug, Deserialize)]
pub struct TestUrlRequest {
    pub protocol_type: String,
    pub stream_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl TestUrlRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.stream_url.trim().is_empty() {
            return Err("Stream URL is required".into());
        }
        if self.stream_url.len() > 2048 {
            return Err("URL too long".into());
        }
        Ok(())
    }
}

/// Test connection to a raw URL (before adding camera)
async fn test_url(Json(req): Json<TestUrlRequest>) -> ApiResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|e| open_nvr_domain::errors::DomainError::Validation(e))?;

    let mut ingester = create_stream_ingester(
        &req.protocol_type,
        &req.stream_url,
        req.username.as_deref(),
        req.password.as_deref(),
    )
    .map_err(|e| open_nvr_domain::errors::DomainError::CameraConnection(e.to_string()))?;

    match tokio::time::timeout(std::time::Duration::from_secs(10), ingester.connect()).await {
        Ok(Ok(())) => {
            let info = ingester.stream_info();
            let _ = ingester.disconnect().await;
            Ok(Json(serde_json::json!({
                "status": "ok",
                "stream_info": info.map(|i| serde_json::json!({
                    "video_codec": i.video_codec,
                    "audio_codec": i.audio_codec,
                })),
            })))
        }
        Ok(Err(e)) => Ok(Json(serde_json::json!({
            "status": "error",
            "error": e.to_string(),
        }))),
        Err(_) => Ok(Json(serde_json::json!({
            "status": "timeout",
            "error": "Connection timed out",
        }))),
    }
}

/// Start recording for a camera
async fn start_camera(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let camera = state.camera_queries.get_camera(id).await?;

    if let Some(ref mgr) = state.camera_manager {
        mgr.start_camera(&camera).await;
        Ok(Json(serde_json::json!({
            "status": "started",
            "camera_id": id,
        })))
    } else {
        Ok(Json(serde_json::json!({
            "status": "error",
            "error": "Camera manager not initialized",
        })))
    }
}

/// Stop recording for a camera
async fn stop_camera(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    if let Some(ref mgr) = state.camera_manager {
        mgr.stop_camera(id).await;
        Ok(Json(serde_json::json!({
            "status": "stopped",
            "camera_id": id,
        })))
    } else {
        Ok(Json(serde_json::json!({
            "status": "error",
            "error": "Camera manager not initialized",
        })))
    }
}

/// Get a snapshot/thumbnail from a camera.
/// This attempts to grab a single frame from the camera's stream URL.
/// For MJPEG cameras, it fetches a single JPEG frame.
/// For RTSP cameras, it returns camera metadata (full snapshot requires ffmpeg).
async fn get_snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<axum::response::Response> {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    let camera = state.camera_queries.get_camera(id).await?;

    // For MJPEG cameras, try to fetch a snapshot directly via HTTP
    if camera.protocol_type.to_string() == "mjpeg" {
        // Attempt to fetch a single JPEG frame from the MJPEG stream URL
        let snapshot_url = camera.stream_url.replace("/video", "/snapshot")
            .replace("/mjpeg", "/snapshot");

        match reqwest::Client::new()
            .get(&snapshot_url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(bytes) = resp.bytes().await {
                    return Ok((
                        StatusCode::OK,
                        [
                            (header::CONTENT_TYPE, "image/jpeg"),
                            (header::CACHE_CONTROL, "no-cache"),
                        ],
                        bytes.to_vec(),
                    ).into_response());
                }
            }
            _ => {
                // Fall through to metadata response
            }
        }
    }

    // For RTSP and other protocols, or if MJPEG snapshot failed,
    // return camera metadata as a JSON response.
    // Full frame capture from RTSP requires ffmpeg integration.
    let response = serde_json::json!({
        "camera_id": id,
        "name": camera.name,
        "protocol": camera.protocol_type.to_string(),
        "status": camera.status.to_string(),
        "stream_url": camera.stream_url,
        "snapshot_available": false,
        "message": "Snapshot capture requires ffmpeg integration for RTSP streams. Use the MJPEG protocol for direct snapshot support.",
    });

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        serde_json::to_vec(&response).unwrap_or_default(),
    ).into_response())
}
