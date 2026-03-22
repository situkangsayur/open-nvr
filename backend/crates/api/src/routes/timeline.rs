use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use open_nvr_application::queries::TimelineResponse;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/timeline", get(get_timeline))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct TimelineQuery {
    camera_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

async fn get_timeline(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> ApiResult<Json<TimelineResponse>> {
    let timeline = state
        .timeline_queries
        .get_timeline(query.camera_id, query.start, query.end)
        .await?;
    Ok(Json(timeline))
}
