use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use open_nvr_domain::entities::DetectionEvent;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/events", get(list_events))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct EventQuery {
    camera_id: Option<Uuid>,
    event_type: Option<String>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<EventQuery>,
) -> ApiResult<Json<Vec<DetectionEvent>>> {
    let end = query.end.unwrap_or_else(Utc::now);
    let start = query.start.unwrap_or_else(|| end - Duration::hours(24));

    let events = if let Some(camera_id) = query.camera_id {
        state.event_repo.find_by_camera(camera_id, start, end).await?
    } else if let Some(ref event_type) = query.event_type {
        let et = event_type
            .parse()
            .map_err(|e: String| open_nvr_domain::errors::DomainError::Validation(e))?;
        state.event_repo.find_by_type(et, start, end).await?
    } else {
        // Default: return recent events for all cameras
        // Use a short time window to prevent huge queries
        let recent_start = end - Duration::hours(1);
        state
            .event_repo
            .find_by_camera(Uuid::nil(), recent_start, end)
            .await
            .unwrap_or_default()
    };

    Ok(Json(events))
}
