use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::broadcast;
use tracing::info;

use crate::state::AppState;

/// Real-time events WebSocket.
/// Broadcasts camera status changes and detection events to connected clients.
pub async fn ws_events_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    let status_rx = state.status_tx.subscribe();
    ws.on_upgrade(move |socket| handle_events(socket, status_rx))
}

async fn handle_events(
    mut socket: WebSocket,
    mut status_rx: broadcast::Receiver<String>,
) {
    info!("WebSocket events client connected");

    let mut ping_interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                let ping = serde_json::json!({
                    "type": "ping",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                if socket.send(Message::Text(ping.to_string().into())).await.is_err() {
                    break;
                }
            }
            result = status_rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "WebSocket events client lagging, skipped messages");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        }
    }

    info!("WebSocket events client disconnected");
}
