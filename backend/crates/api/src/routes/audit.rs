use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use open_nvr_application::dto::*;
use open_nvr_domain::ports::{AuditRepository, NetworkEventRepository};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/audit/logs", get(list_audit_logs))
        .route("/audit/network-events", get(list_network_events))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct AuditQuery {
    limit: Option<i64>,
    user_id: Option<String>,
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

async fn list_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> ApiResult<Json<Vec<AuditLogResponse>>> {
    let logs = if let (Some(user_id), Some(start), Some(end)) = (&query.user_id, query.start, query.end) {
        state.audit_repo.find_by_user(user_id, start, end).await?
    } else {
        state.audit_repo.find_recent(query.limit.unwrap_or(50)).await?
    };

    let response: Vec<AuditLogResponse> = logs.into_iter().map(|l| l.into()).collect();
    Ok(Json(response))
}

async fn list_network_events(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<NetworkEventResponse>>> {
    let events = state.network_event_repo.find_unresolved().await?;
    let response: Vec<NetworkEventResponse> = events.into_iter().map(|e| e.into()).collect();
    Ok(Json(response))
}
