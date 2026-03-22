use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::broadcast;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// WebSocket handler for live camera streaming.
/// Clients connect to /ws/stream/:camera_id and receive raw video/audio frames.
pub async fn ws_stream_handler(
    ws: WebSocketUpgrade,
    Path(camera_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    info!(camera_id = %camera_id, "WebSocket stream connection requested");

    let live_rx = state.live_tx.subscribe();

    ws.on_upgrade(move |socket| handle_stream(socket, camera_id, live_rx))
}

async fn handle_stream(
    mut socket: WebSocket,
    camera_id: Uuid,
    mut live_rx: broadcast::Receiver<open_nvr_worker::recording::LiveFrame>,
) {
    info!(camera_id = %camera_id, "WebSocket stream connected");

    loop {
        match live_rx.recv().await {
            Ok(frame) => {
                // Only send frames for this camera
                if frame.camera_id != camera_id {
                    continue;
                }

                // Send binary frame data
                if socket.send(Message::Binary(frame.data.into())).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!(
                    camera_id = %camera_id,
                    skipped = n,
                    "WebSocket client lagging, skipped frames"
                );
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }

    info!(camera_id = %camera_id, "WebSocket stream disconnected");
}
