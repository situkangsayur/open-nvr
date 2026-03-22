mod camera;
mod group;
mod health;
mod audit;
mod storage;

use axum::Router;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(health::routes())
        .nest("/api", api_routes(state))
}

fn api_routes(state: AppState) -> Router {
    Router::new()
        .merge(camera::routes(state.clone()))
        .merge(group::routes(state.clone()))
        .merge(audit::routes(state.clone()))
        .merge(storage::routes())
}
