use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/recordings/files/{camera_id}", get(list_recording_files))
        .route(
            "/recordings/files/{camera_id}/{filename}",
            get(download_recording),
        )
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct RecordingFile {
    filename: String,
    camera_id: String,
    date: String,
    size_bytes: u64,
    path: String,
    download_url: String,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    date: Option<String>, // YYYY-MM-DD
}

async fn list_recording_files(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Response {
    let rec_base = match &state.hls_manager {
        Some(mgr) => mgr.recordings_dir().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Recording not available"})),
            )
                .into_response()
        }
    };

    let rec_dir = rec_base.join(camera_id.to_string());

    let mut files = Vec::new();

    // List date directories
    let mut dir = match tokio::fs::read_dir(&rec_dir).await {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({"files": [], "count": 0})),
            )
                .into_response()
        }
    };

    let target_date = query.date.clone();

    while let Ok(Some(entry)) = dir.next_entry().await {
        let date_name = entry.file_name().to_string_lossy().to_string();

        if let Some(ref td) = target_date {
            if &date_name != td {
                continue;
            }
        }

        if let Ok(mut date_dir) = tokio::fs::read_dir(entry.path()).await {
            while let Ok(Some(file_entry)) = date_dir.next_entry().await {
                let fname = file_entry.file_name().to_string_lossy().to_string();
                if fname.ends_with(".mp4") {
                    let size_bytes = file_entry
                        .metadata()
                        .await
                        .map(|m| m.len())
                        .unwrap_or(0);
                    files.push(RecordingFile {
                        filename: fname.clone(),
                        camera_id: camera_id.to_string(),
                        date: date_name.clone(),
                        size_bytes,
                        path: format!("{}/{}", date_name, fname),
                        download_url: format!(
                            "/api/recordings/files/{}/{}",
                            camera_id, fname
                        ),
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| b.filename.cmp(&a.filename));

    let count = files.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({"files": files, "count": count})),
    )
        .into_response()
}

async fn download_recording(
    State(state): State<AppState>,
    Path((camera_id, filename)): Path<(Uuid, String)>,
) -> Response {
    // Security: prevent path traversal
    if filename.contains("..") || filename.contains('/') {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let rec_base = match &state.hls_manager {
        Some(mgr) => mgr.recordings_dir().clone(),
        None => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };

    let rec_dir = rec_base.join(camera_id.to_string());

    // Search through date directories to find the file
    if let Ok(mut dates) = tokio::fs::read_dir(&rec_dir).await {
        while let Ok(Some(date_entry)) = dates.next_entry().await {
            let file_path = date_entry.path().join(&filename);
            if file_path.exists() {
                match tokio::fs::read(&file_path).await {
                    Ok(data) => {
                        return (
                            StatusCode::OK,
                            [
                                (header::CONTENT_TYPE, "video/mp4"),
                                (
                                    header::CONTENT_DISPOSITION,
                                    &format!("attachment; filename=\"{}\"", filename),
                                ),
                            ],
                            data,
                        )
                            .into_response();
                    }
                    Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                }
            }
        }
    }

    StatusCode::NOT_FOUND.into_response()
}
