use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::Response;
use tracing::info;

/// Real-time events WebSocket.
/// Broadcasts camera status changes and detection events to connected clients.
pub async fn ws_events_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_events)
}

async fn handle_events(mut socket: WebSocket) {
    info!("WebSocket events client connected");

    // For now, send a keepalive ping every 30 seconds
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        interval.tick().await;
        let ping = serde_json::json!({
            "type": "ping",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        if socket
            .send(Message::Text(ping.to_string().into()))
            .await
            .is_err()
        {
            break;
        }
    }

    info!("WebSocket events client disconnected");
}
