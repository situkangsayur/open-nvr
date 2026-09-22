//! Low-latency live view. Browsers and the Android WebView speak go2rtc's MSE
//! protocol (JSON control messages + binary fMP4 fragments) over a WebSocket;
//! this route authenticates the viewer (the /api auth layer, token in the
//! query string) and relays that socket to go2rtc on loopback.

use axum::extract::ws::{Message as AxMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use futures_util::{SinkExt, StreamExt};
use open_nvr_worker::go2rtc::{self, Go2Rtc};
use serde::Deserialize;
use tokio_tungstenite::tungstenite::Message as TgMessage;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/live/{camera_id}/ws", get(live_ws))
        .with_state(state)
}

/// Standalone player page for WebView clients. Public: it holds no data, the
/// token travels in the URL fragment and is only used for the WebSocket.
pub fn public_routes() -> Router {
    Router::new()
        .route("/live-player.html", get(player_html))
        .route("/live-player.js", get(player_js))
}

#[derive(Debug, Deserialize)]
struct LiveQuery {
    quality: Option<String>,
}

async fn live_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Query(q): Query<LiveQuery>,
) -> Response {
    if state.go2rtc.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Low-latency live is not enabled on this server" })),
        )
            .into_response();
    }
    let camera = match state.camera_queries.get_camera(camera_id).await {
        Ok(c) => c,
        Err(e) => return crate::error::ApiError::from(e).into_response(),
    };
    let sub = q.quality.as_deref() == Some("sub");
    let src = Go2Rtc::stream_name(&camera, sub);

    ws.on_upgrade(move |socket| relay(socket, src))
}

async fn relay(client: WebSocket, src: String) {
    let url = format!("ws://{}/api/ws?src={}", go2rtc::API_ADDR, src);
    let upstream = match tokio_tungstenite::connect_async(&url).await {
        Ok((s, _)) => s,
        Err(e) => {
            tracing::warn!(src = %src, error = %e, "Cannot reach go2rtc");
            let mut client = client;
            let _ = client
                .send(AxMessage::Text(
                    r#"{"type":"error","value":"live service unavailable"}"#.into(),
                ))
                .await;
            return;
        }
    };

    let (mut up_tx, mut up_rx) = upstream.split();
    let (mut cl_tx, mut cl_rx) = client.split();

    let to_upstream = async {
        while let Some(Ok(msg)) = cl_rx.next().await {
            let out = match msg {
                AxMessage::Text(t) => TgMessage::Text(t.as_str().into()),
                AxMessage::Binary(b) => TgMessage::Binary(b),
                AxMessage::Ping(p) => TgMessage::Ping(p),
                AxMessage::Pong(p) => TgMessage::Pong(p),
                AxMessage::Close(_) => break,
            };
            if up_tx.send(out).await.is_err() {
                break;
            }
        }
        let _ = up_tx.close().await;
    };

    let to_client = async {
        while let Some(Ok(msg)) = up_rx.next().await {
            let out = match msg {
                TgMessage::Text(t) => AxMessage::Text(t.as_str().into()),
                TgMessage::Binary(b) => AxMessage::Binary(b),
                TgMessage::Ping(p) => AxMessage::Ping(p),
                TgMessage::Pong(p) => AxMessage::Pong(p),
                TgMessage::Close(_) => break,
                TgMessage::Frame(_) => continue,
            };
            if cl_tx.send(out).await.is_err() {
                break;
            }
        }
        let _ = cl_tx.close().await;
    };

    // Whichever side goes away first ends the session.
    tokio::select! {
        _ = to_upstream => {},
        _ = to_client => {},
    }
}

async fn player_html(headers: axum::http::HeaderMap) -> Response {
    // Older WebViews don't let 'self' cover ws:/wss:, so name the host too.
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .filter(|h| h.chars().all(|c| c.is_ascii_alphanumeric() || ".:-[]".contains(c)))
        .unwrap_or("");
    let ws_src = if host.is_empty() { String::new() } else { format!(" ws://{host} wss://{host}") };
    let csp = format!(
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
         media-src 'self' blob:; connect-src 'self'{ws_src}; img-src 'self' data:; frame-ancestors 'none'"
    );
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8".to_string()),
            (header::CONTENT_SECURITY_POLICY, csp),
        ],
        include_str!("../../static/live-player.html"),
    )
        .into_response()
}

async fn player_js() -> Response {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        include_str!("../../static/live-player.js"),
    )
        .into_response()
}
