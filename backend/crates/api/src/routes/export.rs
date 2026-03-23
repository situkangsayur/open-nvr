use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/recordings/export", get(export_recording))
        .route("/recordings/export/manifest", get(export_manifest))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct ExportQuery {
    camera_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// GET /api/recordings/export — Download MP4 via ffmpeg direct capture
/// This grabs video directly from the camera's RTSP stream for the requested duration
async fn export_recording(
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> Response {
    // Check ffmpeg
    if tokio::process::Command::new("ffmpeg").arg("-version").output().await.is_err() {
        return (StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(serde_json::json!({"error": "ffmpeg not installed"}))).into_response();
    }

    // Get camera
    let camera = match state.camera_queries.get_camera(query.camera_id).await {
        Ok(c) => c,
        Err(e) => return (StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    // Calculate duration
    let duration_secs = (query.end - query.start).num_seconds().max(1).min(3600); // Max 1 hour

    // Use ffmpeg to capture from RTSP and output MP4
    let output = match tokio::process::Command::new("ffmpeg")
        .args([
            "-rtsp_transport", "tcp",
            "-i", &camera.stream_url,
            "-t", &duration_secs.to_string(),
            "-c:v", "copy",
            "-c:a", "copy",
            "-movflags", "+faststart",
            "-f", "mp4",
            "pipe:1",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({"error": format!("ffmpeg failed: {}", e)}))).into_response(),
    };

    if output.status.success() && !output.stdout.is_empty() {
        let filename = format!(
            "recording_{}_{}_{}.mp4",
            camera.name.replace(' ', "_"),
            query.start.format("%Y%m%d_%H%M%S"),
            query.end.format("%H%M%S"),
        );

        (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "video/mp4"),
                (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", filename)),
            ],
            output.stdout,
        ).into_response()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!(error = %stderr, "ffmpeg export failed");
        (StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({
                "error": "Export failed - camera may be unreachable",
                "detail": stderr.chars().take(500).collect::<String>(),
            }))).into_response()
    }
}

/// GET /api/recordings/export/manifest — Get segment URLs for client-side download
async fn export_manifest(
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> Response {
    let recordings = match state.recording_queries
        .list_by_camera(query.camera_id, query.start, query.end)
        .await
    {
        Ok(r) => r,
        Err(e) => return (StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    let mut all_segment_urls = Vec::new();
    for rec in &recordings {
        if let Ok(segments) = state.recording_queries.get_segments_with_urls(rec.id).await {
            for seg in segments {
                if let Some(url) = seg.download_url {
                    all_segment_urls.push(url);
                }
            }
        }
    }

    let manifest = serde_json::json!({
        "camera_id": query.camera_id,
        "start": query.start,
        "end": query.end,
        "recording_count": recordings.len(),
        "segment_count": all_segment_urls.len(),
        "segments": all_segment_urls,
    });

    (StatusCode::OK, axum::Json(manifest)).into_response()
}
