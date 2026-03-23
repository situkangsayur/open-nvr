use axum::extract::{Query, State};
use axum::http::header;
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
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct ExportQuery {
    camera_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// Export recordings for a camera within a time range.
/// Returns concatenated segments as a downloadable file.
/// NOTE: For proper MP4 muxing, ffmpeg would be needed.
/// This implementation returns segment data directly.
async fn export_recording(
    State(state): State<AppState>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, crate::error::ApiError> {
    // Get recordings in time range
    let recordings = state.recording_queries
        .list_by_camera(query.camera_id, query.start, query.end)
        .await?;

    if recordings.is_empty() {
        return Err(open_nvr_domain::errors::DomainError::NotFound {
            entity_type: "recording".into(),
            id: query.camera_id,
        }.into());
    }

    // Get all segments with download URLs
    let mut all_segment_urls = Vec::new();
    for rec in &recordings {
        let segments = state.recording_queries
            .get_segments_with_urls(rec.id)
            .await?;
        for seg in segments {
            if let Some(url) = seg.download_url {
                all_segment_urls.push(url);
            }
        }
    }

    // Return a JSON manifest with segment URLs for client-side download
    // (True MP4 concatenation would require ffmpeg on the server)
    let filename = format!(
        "export_{}_{}.json",
        query.camera_id.to_string().split('-').next().unwrap_or("cam"),
        query.start.format("%Y%m%d_%H%M%S"),
    );

    let manifest = serde_json::json!({
        "camera_id": query.camera_id,
        "start": query.start,
        "end": query.end,
        "recording_count": recordings.len(),
        "segment_count": all_segment_urls.len(),
        "segments": all_segment_urls,
        "note": "Download segments individually or use ffmpeg to concatenate: ffmpeg -i 'concat:seg1.mp4|seg2.mp4' -c copy output.mp4"
    });

    let content_disposition = format!("attachment; filename=\"{}\"", filename);

    Ok((
        [
            (header::CONTENT_TYPE, "application/json".to_string()),
            (header::CONTENT_DISPOSITION, content_disposition),
        ],
        serde_json::to_string_pretty(&manifest).unwrap_or_default(),
    ).into_response())
}
