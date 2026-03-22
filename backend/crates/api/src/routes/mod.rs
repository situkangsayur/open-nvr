mod camera;
mod group;
mod health;
mod audit;
mod storage;

use axum::routing::get;
use axum::Router;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let ws_routes = Router::new()
        .route("/ws/stream/{camera_id}", get(crate::ws::stream::ws_stream_handler))
        .with_state(state.clone());

    Router::new()
        .merge(health::routes())
        .merge(ws_routes)
        .nest("/api", api_routes(state))
}

fn api_routes(state: AppState) -> Router {
    Router::new()
        .merge(camera::routes(state.clone()))
        .merge(group::routes(state.clone()))
        .merge(audit::routes(state.clone()))
        .merge(storage::routes())
}
