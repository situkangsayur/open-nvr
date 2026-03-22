use axum::routing::post;
use axum::{Json, Router};
use open_nvr_application::dto::*;
use open_nvr_application::queries::StorageQueryService;

pub fn routes() -> Router {
    Router::new()
        .route("/settings/storage/estimate", post(estimate_storage))
}

async fn estimate_storage(
    Json(req): Json<StorageEstimateRequest>,
) -> Json<StorageEstimate> {
    Json(StorageQueryService::estimate(&req))
}
