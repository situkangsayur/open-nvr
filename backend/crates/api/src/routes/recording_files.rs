//! The MP4 archive written by the ffmpeg recorders:
//! `<RECORDINGS_DIR>/<camera_id>/<YYYY-MM-DD>/<YYYYmmdd_HHMMSS>.mp4`, UTC,
//! one file per ~5 minutes.

use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tower::ServiceExt;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/recordings/files/{camera_id}", get(list_recording_files))
        .route("/recordings/files/{camera_id}/days", get(list_days))
        .route(
            "/recordings/files/{camera_id}/{filename}",
            get(download_recording),
        )
        .with_state(state)
}

/// A file modified this recently may still be open in ffmpeg, and with
/// `+faststart` its index is only written when the segment closes.
const IN_PROGRESS_WINDOW_SECS: i64 = 20;
/// Longest a single segment can plausibly span.
const MAX_SEGMENT_SECS: i64 = 900;

#[derive(Debug, Serialize)]
struct RecordingFile {
    filename: String,
    camera_id: String,
    /// UTC directory the file lives in (kept for older clients).
    date: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    duration_secs: i64,
    size_bytes: u64,
    /// False while ffmpeg is still writing it (not playable yet).
    complete: bool,
    path: String,
    url: String,
    download_url: String,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    date: Option<String>, // YYYY-MM-DD (UTC directory)
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct DaysQuery {
    /// Minutes east of UTC the client wants days grouped by (WIB = 420).
    tz_offset_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct DownloadQuery {
    download: Option<String>,
}

fn camera_dir(state: &AppState, camera_id: Uuid) -> Option<PathBuf> {
    state
        .hls_manager
        .as_ref()
        .map(|m| m.recordings_dir().join(camera_id.to_string()))
}

fn unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "Recording not available"})),
    )
        .into_response()
}

fn parse_start(filename: &str) -> Option<DateTime<Utc>> {
    let stem = filename.strip_suffix(".mp4")?;
    NaiveDateTime::parse_from_str(stem, "%Y%m%d_%H%M%S")
        .ok()
        .map(|n| Utc.from_utc_datetime(&n))
}

/// Every recording of one camera, sorted by start time.
async fn scan(dir: &std::path::Path, camera_id: Uuid) -> Vec<RecordingFile> {
    let mut files = Vec::new();
    let Ok(mut dates) = tokio::fs::read_dir(dir).await else {
        return files;
    };
    let now = Utc::now();

    while let Ok(Some(date_entry)) = dates.next_entry().await {
        let date = date_entry.file_name().to_string_lossy().to_string();
        let Ok(mut entries) = tokio::fs::read_dir(date_entry.path()).await else {
            continue;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let filename = entry.file_name().to_string_lossy().to_string();
            let Some(start) = parse_start(&filename) else { continue };
            let Ok(meta) = entry.metadata().await else { continue };
            let modified: DateTime<Utc> = meta
                .modified()
                .map(DateTime::<Utc>::from)
                .unwrap_or(start);
            let complete = (now - modified).num_seconds() > IN_PROGRESS_WINDOW_SECS;
            // The last write is the end of the footage, give or take a GOP.
            let end = modified.clamp(start, start + Duration::seconds(MAX_SEGMENT_SECS));
            let url = format!("/api/recordings/files/{}/{}", camera_id, filename);
            files.push(RecordingFile {
                path: format!("{}/{}", date, filename),
                download_url: format!("{}?download=1", url),
                url,
                camera_id: camera_id.to_string(),
                date: date.clone(),
                duration_secs: (end - start).num_seconds(),
                size_bytes: meta.len(),
                filename,
                start,
                end,
                complete,
            });
        }
    }

    files.sort_by_key(|f| f.start);
    files
}

async fn list_recording_files(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Response {
    let Some(dir) = camera_dir(&state, camera_id) else { return unavailable() };
    let mut files = scan(&dir, camera_id).await;

    if let Some(ref date) = query.date {
        files.retain(|f| &f.date == date);
    }
    // Overlap, not containment: a segment that started before `from` but
    // runs into the range is part of it.
    if let Some(from) = query.from {
        files.retain(|f| f.end > from);
    }
    if let Some(to) = query.to {
        files.retain(|f| f.start < to);
    }

    let count = files.len();
    Json(serde_json::json!({"files": files, "count": count})).into_response()
}

async fn list_days(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Query(query): Query<DaysQuery>,
) -> Response {
    let Some(dir) = camera_dir(&state, camera_id) else { return unavailable() };
    let offset_secs = query.tz_offset_minutes.unwrap_or(0).clamp(-14 * 60, 14 * 60) * 60;
    let tz = FixedOffset::east_opt(offset_secs).unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());

    let mut days: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    for f in scan(&dir, camera_id).await {
        let day = f.start.with_timezone(&tz).format("%Y-%m-%d").to_string();
        let e = days.entry(day).or_default();
        e.0 += 1;
        e.1 += f.size_bytes;
    }

    let days: Vec<_> = days
        .into_iter()
        .rev()
        .map(|(date, (count, size))| serde_json::json!({"date": date, "count": count, "size_bytes": size}))
        .collect();
    Json(serde_json::json!({"days": days})).into_response()
}

/// Stream one recording. Range requests are honoured so players can seek
/// without downloading the whole file.
async fn download_recording(
    State(state): State<AppState>,
    Path((camera_id, filename)): Path<(Uuid, String)>,
    Query(query): Query<DownloadQuery>,
    request: Request,
) -> Response {
    if parse_start(&filename).is_none() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let Some(dir) = camera_dir(&state, camera_id) else { return unavailable() };

    let mut found = None;
    if let Ok(mut dates) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(date_entry)) = dates.next_entry().await {
            let candidate = date_entry.path().join(&filename);
            if tokio::fs::metadata(&candidate).await.map(|m| m.is_file()).unwrap_or(false) {
                found = Some(candidate);
                break;
            }
        }
    }
    let Some(path) = found else { return StatusCode::NOT_FOUND.into_response() };

    let mut response = match tower_http::services::ServeFile::new(&path)
        .oneshot(request)
        .await
    {
        Ok(r) => r.map(axum::body::Body::new),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let disposition = if query.download.is_some() { "attachment" } else { "inline" };
    if let Ok(v) = HeaderValue::from_str(&format!("{}; filename=\"{}\"", disposition, filename)) {
        response.headers_mut().insert(header::CONTENT_DISPOSITION, v);
    }
    response
}
