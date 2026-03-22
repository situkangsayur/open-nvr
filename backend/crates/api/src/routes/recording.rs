use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use open_nvr_application::queries::{RecordingResponse, SegmentResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/recordings", get(list_recordings))
        .route("/recordings/{id}", get(get_recording))
        .route("/recordings/{id}/segments", get(get_segments))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct RecordingQuery {
    camera_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

async fn list_recordings(
    State(state): State<AppState>,
    Query(query): Query<RecordingQuery>,
) -> ApiResult<Json<Vec<RecordingResponse>>> {
    let recordings = state
        .recording_queries
        .list_by_camera(query.camera_id, query.start, query.end)
        .await?;
    Ok(Json(recordings))
}

async fn get_recording(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RecordingResponse>> {
    let recording = state.recording_queries.get_recording(id).await?;
    Ok(Json(recording))
}

async fn get_segments(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<SegmentResponse>>> {
    let segments = state.recording_queries.get_segments_with_urls(id).await?;
    Ok(Json(segments))
}
