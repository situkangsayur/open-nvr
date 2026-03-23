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
        .route("/cameras/{id}/ping", post(ping_camera))
        .route("/cameras/ping-all", post(ping_all_cameras))
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
/// For MJPEG cameras, it fetches a single JPEG frame via HTTP.
/// For RTSP and other protocols, it uses ffmpeg to grab a single frame.
async fn get_snapshot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    let camera = match state.camera_queries.get_camera(id).await {
        Ok(c) => c,
        Err(e) => {
            let err: crate::error::ApiError = e.into();
            return err.into_response();
        }
    };

    // For MJPEG cameras, try to fetch a snapshot directly via HTTP first
    if camera.protocol_type.to_string() == "mjpeg" {
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
                    return (
                        StatusCode::OK,
                        [
                            (header::CONTENT_TYPE, "image/jpeg"),
                            (header::CACHE_CONTROL, "no-cache"),
                        ],
                        bytes.to_vec(),
                    ).into_response();
                }
            }
            _ => {
                // Fall through to ffmpeg capture
            }
        }
    }

    // Check if ffmpeg is available
    if tokio::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .await
        .is_err()
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"error": "ffmpeg not installed on server"})),
        ).into_response();
    }

    // Use ffmpeg to grab a single frame from the camera stream
    let output = tokio::process::Command::new("ffmpeg")
        .args([
            "-rtsp_transport", "tcp",
            "-i", &camera.stream_url,
            "-frames:v", "1",
            "-f", "image2",
            "-c:v", "mjpeg",
            "-q:v", "2",
            "pipe:1",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() && !out.stdout.is_empty() => {
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "image/jpeg"),
                    (header::CACHE_CONTROL, "no-cache"),
                ],
                out.stdout,
            ).into_response()
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            tracing::error!(
                camera_id = %id,
                exit_code = ?out.status.code(),
                stderr = %stderr,
                "ffmpeg snapshot failed"
            );
            (
                StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({
                    "error": format!("Snapshot failed: ffmpeg exit code {:?}", out.status.code()),
                    "camera_id": id,
                })),
            ).into_response()
        }
        Err(e) => {
            tracing::error!(camera_id = %id, error = %e, "ffmpeg execution error");
            (
                StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({
                    "error": format!("ffmpeg execution error: {}", e),
                    "camera_id": id,
                })),
            ).into_response()
        }
    }
}

/// Ping a camera to check if it's still alive (TCP probe on known ports)
async fn ping_camera(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let camera = state.camera_queries.get_camera(id).await?;

    // Extract host from stream URL
    let host = extract_host(&camera.stream_url);
    let port = extract_port(&camera.stream_url).unwrap_or(554);

    let addr = format!("{}:{}", host, port);
    let alive = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    };

    // Update camera status based on ping result
    let new_status = if alive {
        open_nvr_domain::entities::CameraStatus::Online
    } else {
        open_nvr_domain::entities::CameraStatus::Offline
    };

    // Update status in DB
    let _ = sqlx::query("UPDATE cameras SET status = $1, updated_at = now() WHERE id = $2")
        .bind(new_status.to_string())
        .bind(id)
        .execute(&state.db_pool)
        .await;

    // Broadcast status change via WebSocket
    let event = serde_json::json!({
        "type": "camera_status",
        "camera_id": id.to_string(),
        "name": camera.name,
        "status": new_status.to_string(),
        "alive": alive,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let _ = state.status_tx.send(event.to_string());

    Ok(Json(serde_json::json!({
        "camera_id": id,
        "name": camera.name,
        "alive": alive,
        "status": new_status.to_string(),
        "host": host,
        "port": port,
    })))
}

/// Ping all cameras to update their liveness status
async fn ping_all_cameras(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let cameras = state.camera_queries.list_cameras().await?;
    let mut results = Vec::new();

    for camera in &cameras {
        let host = extract_host(&camera.stream_url);
        let port = extract_port(&camera.stream_url).unwrap_or(554);
        let addr = format!("{}:{}", host, port);

        let alive = match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => true,
            _ => false,
        };

        let new_status = if alive { "online" } else { "offline" };
        let _ = sqlx::query("UPDATE cameras SET status = $1, updated_at = now() WHERE id = $2")
            .bind(new_status)
            .bind(camera.id)
            .execute(&state.db_pool)
            .await;

        // Broadcast status change via WebSocket
        let event = serde_json::json!({
            "type": "camera_status",
            "camera_id": camera.id.to_string(),
            "name": camera.name,
            "status": new_status,
            "alive": alive,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let _ = state.status_tx.send(event.to_string());

        results.push(serde_json::json!({
            "camera_id": camera.id,
            "name": camera.name,
            "alive": alive,
            "status": new_status,
        }));
    }

    Ok(Json(results))
}

fn extract_host(url: &str) -> String {
    // Parse host from rtsp://user:pass@host:port/path or http://host:port/path
    let without_scheme = url.split("://").nth(1).unwrap_or(url);
    let without_auth = without_scheme.split('@').last().unwrap_or(without_scheme);
    let host_port = without_auth.split('/').next().unwrap_or(without_auth);
    host_port.split(':').next().unwrap_or(host_port).to_string()
}

fn extract_port(url: &str) -> Option<u16> {
    let without_scheme = url.split("://").nth(1)?;
    let without_auth = without_scheme.split('@').last()?;
    let host_port = without_auth.split('/').next()?;
    let port_str = host_port.split(':').nth(1)?;
    port_str.parse().ok()
}
