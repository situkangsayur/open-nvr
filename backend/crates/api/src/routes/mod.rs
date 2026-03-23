mod camera;
mod camera_ops;
mod discovery;
mod events;
mod group;
mod health;
mod audit;
mod storage;
mod recording;
mod timeline;
mod zones;
mod ptz;
mod layout;
mod retention;

use axum::routing::get;
use axum::Router;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let ws_routes = Router::new()
        .route("/ws/stream/{camera_id}", get(crate::ws::stream::ws_stream_handler))
        .route("/ws/events", get(crate::ws::events::ws_events_handler))
        .with_state(state.clone());

    Router::new()
        .merge(health::routes())
        .merge(ws_routes)
        .nest("/api", api_routes(state))
}

fn api_routes(state: AppState) -> Router {
    Router::new()
        .merge(camera::routes(state.clone()))
        .merge(camera_ops::routes(state.clone()))
        .merge(discovery::routes(state.clone()))
        .merge(group::routes(state.clone()))
        .merge(audit::routes(state.clone()))
        .merge(storage::routes())
        .merge(recording::routes(state.clone()))
        .merge(timeline::routes(state.clone()))
        .merge(zones::routes(state.clone()))
        .merge(events::routes(state.clone()))
        .merge(ptz::routes(state.clone()))
        .merge(layout::routes(state.clone()))
        .merge(retention::routes(state.clone()))
}
