use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use open_nvr_domain::entities::Camera;
use serde::Deserialize;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/cameras/{id}/ptz", get(ptz_support).post(ptz_command))
        .route("/cameras/{id}/ptz/preset", post(goto_preset))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct PtzCommand {
    pub action: String,      // pan_left, pan_right, tilt_up, tilt_down, zoom_in, zoom_out, stop, home
    pub speed: Option<f32>,  // 0.0-1.0
}

impl PtzCommand {
    pub fn validate(&self) -> Result<(), String> {
        let valid_actions = ["pan_left", "pan_right", "tilt_up", "tilt_down",
                            "zoom_in", "zoom_out", "stop", "home"];
        if !valid_actions.contains(&self.action.as_str()) {
            return Err(format!("Invalid PTZ action: {}. Valid: {:?}", self.action, valid_actions));
        }
        if let Some(speed) = self.speed {
            if !(0.0..=1.0).contains(&speed) {
                return Err("Speed must be between 0.0 and 1.0".into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct PresetCommand {
    pub preset_id: u32,
}

impl PresetCommand {
    pub fn validate(&self) -> Result<(), String> {
        if self.preset_id > 255 {
            return Err("Preset ID must be 0-255".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ONVIF discovery
// ---------------------------------------------------------------------------

/// Where to send PTZ SOAP calls for one camera.
#[derive(Debug, Clone)]
struct PtzEndpoint {
    ptz_url: String,
    profile_token: String,
}

/// Discovery result per camera. Negative results are cached for a while so a
/// camera without ONVIF is not probed on every button press.
#[derive(Clone)]
enum Discovery {
    Found(PtzEndpoint),
    Missing(Instant),
}

const RETRY_MISSING_AFTER: Duration = Duration::from_secs(300);
/// Ports ONVIF commonly listens on across cheap and brand-name cameras.
const ONVIF_PORTS: [u16; 6] = [8899, 80, 8000, 2020, 8080, 5000];

fn cache() -> &'static RwLock<HashMap<Uuid, Discovery>> {
    static CACHE: OnceLock<RwLock<HashMap<Uuid, Discovery>>> = OnceLock::new();
    CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn http() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(6))
            .connect_timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_default()
    })
}

fn credentials(camera: &Camera) -> Option<(String, String)> {
    static ENC: OnceLock<Option<open_nvr_infrastructure::crypto::credentials::CredentialEncryptor>> =
        OnceLock::new();
    let enc = ENC
        .get_or_init(|| open_nvr_infrastructure::crypto::credentials::CredentialEncryptor::from_env().ok())
        .as_ref()?;
    let creds = enc.decrypt(camera.credentials_encrypted.as_ref()?).ok()?;
    let (user, pass) = creds.split_once(':')?;
    Some((user.to_string(), pass.to_string()))
}

/// WS-Security UsernameToken (PasswordDigest), required by cameras that have
/// a password set. Cameras without one ignore the header.
fn security_header(creds: Option<&(String, String)>) -> String {
    let Some((user, pass)) = creds else { return String::new() };
    let nonce = Uuid::new_v4().into_bytes();
    let created = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut hasher = Sha1::new();
    hasher.update(nonce);
    hasher.update(created.as_bytes());
    hasher.update(pass.as_bytes());
    let b64 = base64::engine::general_purpose::STANDARD;
    format!(
        r#"<s:Header><Security s:mustUnderstand="1" xmlns="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd"><UsernameToken><Username>{}</Username><Password Type="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-username-token-profile-1.0#PasswordDigest">{}</Password><Nonce EncodingType="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-soap-message-security-1.0#Base64Binary">{}</Nonce><Created xmlns="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd">{}</Created></UsernameToken></Security></s:Header>"#,
        xml_escape(user),
        b64.encode(hasher.finalize()),
        b64.encode(nonce),
        created
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

async fn soap(url: &str, body: &str, creds: Option<&(String, String)>) -> Result<String, String> {
    let envelope = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:tptz="http://www.onvif.org/ver20/ptz/wsdl" xmlns:tt="http://www.onvif.org/ver10/schema">{}<s:Body>{}</s:Body></s:Envelope>"#,
        security_header(creds),
        body
    );
    let resp = http()
        .post(url)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .body(envelope)
        .send()
        .await
        .map_err(|e| format!("camera unreachable: {}", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if status.is_success() && !text.contains(":Fault>") {
        Ok(text)
    } else if text.contains("NotAuthorized") || status == reqwest::StatusCode::UNAUTHORIZED {
        Err("camera rejected the credentials".into())
    } else {
        Err(format!("camera returned HTTP {}", status.as_u16()))
    }
}

/// Text of the first `<prefix:tag>` element inside `xml` (any namespace prefix).
fn element_text<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let mut rest = xml;
    loop {
        let open = rest.find('<')?;
        rest = &rest[open + 1..];
        let end = rest.find('>')?;
        let name = rest[..end].split_whitespace().next().unwrap_or("");
        let local = name.rsplit(':').next().unwrap_or(name);
        if local == tag && !name.starts_with('/') {
            let body = &rest[end + 1..];
            let close = body.find("</")?;
            return Some(body[..close].trim());
        }
        rest = &rest[end + 1..];
    }
}

/// Every `token="..."` of `<*:Profiles ...>` elements, in document order,
/// each paired with whether the profile carries a PTZ configuration.
fn profiles(xml: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut chunks: Vec<&str> = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find(":Profiles ") {
        rest = &rest[i + 1..];
        chunks.push(rest);
    }
    for (i, chunk) in chunks.iter().enumerate() {
        let end = chunks
            .get(i + 1)
            .map(|next| chunk.len() - next.len())
            .unwrap_or(chunk.len());
        let section = &chunk[..end];
        let token = section
            .split("token=\"")
            .nth(1)
            .and_then(|t| t.split('"').next())
            .unwrap_or("");
        if !token.is_empty() {
            out.push((token.to_string(), section.contains("PTZConfiguration")));
        }
    }
    out
}

fn camera_host(camera: &Camera) -> Option<String> {
    let url = camera.onvif_url.as_deref().unwrap_or(&camera.stream_url);
    let rest = url.split("://").nth(1)?;
    let authority = rest.split('/').next()?;
    let host_port = authority.rsplit('@').next()?;
    let host = host_port.split(':').next()?;
    (!host.is_empty()).then(|| host.to_string())
}

/// What discovery learned about a camera.
struct Probe {
    ptz: Option<PtzEndpoint>,
    /// RTSP URI of a second (lower resolution) profile, if the camera has one.
    sub_stream: Option<String>,
}

async fn probe(camera: &Camera) -> Option<Probe> {
    let host = camera_host(camera)?;
    let creds = credentials(camera);

    // An explicit onvif_url wins; otherwise try the usual ports.
    let device_urls: Vec<String> = match camera.onvif_url {
        Some(ref u) if u.contains("/onvif/") => vec![u.clone()],
        Some(ref u) => vec![format!("{}/onvif/device_service", u.trim_end_matches('/'))],
        None => ONVIF_PORTS
            .iter()
            .map(|p| format!("http://{}:{}/onvif/device_service", host, p))
            .collect(),
    };

    for device_url in device_urls {
        let caps = match soap(
            &device_url,
            r#"<GetCapabilities xmlns="http://www.onvif.org/ver10/device/wsdl"><Category>All</Category></GetCapabilities>"#,
            creds.as_ref(),
        )
        .await
        {
            Ok(c) => c,
            Err(_) => continue,
        };

        let section = |name: &str| -> Option<String> {
            let start = caps.find(&format!(":{}>", name))?;
            element_text(&caps[start..], "XAddr").map(str::to_string)
        };
        let media_url = section("Media");
        let ptz_url = section("PTZ");

        let mut result = Probe { ptz: None, sub_stream: None };
        let Some(media_url) = media_url else { return Some(result) };
        let Ok(profiles_xml) = soap(
            &media_url,
            r#"<GetProfiles xmlns="http://www.onvif.org/ver10/media/wsdl"/>"#,
            creds.as_ref(),
        )
        .await
        else {
            return Some(result);
        };
        let profiles = profiles(&profiles_xml);

        if let Some(ptz_url) = ptz_url {
            if let Some((token, _)) = profiles.iter().find(|(_, has_ptz)| *has_ptz) {
                result.ptz = Some(PtzEndpoint { ptz_url, profile_token: token.clone() });
            }
        }

        if let Some((token, _)) = profiles.get(1) {
            let body = format!(
                r#"<GetStreamUri xmlns="http://www.onvif.org/ver10/media/wsdl"><StreamSetup><Stream xmlns="http://www.onvif.org/ver10/schema">RTP-Unicast</Stream><Transport xmlns="http://www.onvif.org/ver10/schema"><Protocol>RTSP</Protocol></Transport></StreamSetup><ProfileToken>{}</ProfileToken></GetStreamUri>"#,
                xml_escape(token)
            );
            if let Ok(xml) = soap(&media_url, &body, creds.as_ref()).await {
                result.sub_stream = element_text(&xml, "Uri").map(str::to_string);
            }
        }
        return Some(result);
    }
    None
}

async fn endpoint(state: &AppState, camera: &Camera) -> Option<PtzEndpoint> {
    if let Some(entry) = cache().read().await.get(&camera.id) {
        match entry {
            Discovery::Found(ep) => return Some(ep.clone()),
            Discovery::Missing(at) if at.elapsed() < RETRY_MISSING_AFTER => return None,
            Discovery::Missing(_) => {}
        }
    }
    discover(state, camera).await
}

/// Probe one camera, cache the answer and keep the camera row in sync
/// (`ptz_capable`, and `sub_stream_url` when it was empty).
async fn discover(state: &AppState, camera: &Camera) -> Option<PtzEndpoint> {
    let probe = probe(camera).await;
    let ep = probe.as_ref().and_then(|p| p.ptz.clone());

    cache().write().await.insert(
        camera.id,
        match ep {
            Some(ref e) => Discovery::Found(e.clone()),
            None => Discovery::Missing(Instant::now()),
        },
    );

    // Only a camera that answered ONVIF gets its flag rewritten; an offline
    // camera keeps whatever it had.
    if let Some(ref p) = probe {
        let capable = p.ptz.is_some();
        if capable != camera.ptz_capable {
            let _ = sqlx::query("UPDATE cameras SET ptz_capable = $1 WHERE id = $2")
                .bind(capable)
                .bind(camera.id)
                .execute(&state.db_pool)
                .await;
            tracing::info!(camera_id = %camera.id, ptz = capable, "PTZ capability detected");
        }
        if camera.sub_stream_url.is_none() {
            if let Some(ref sub) = p.sub_stream {
                if sub.starts_with("rtsp://") && sub != &camera.stream_url {
                    let _ = sqlx::query("UPDATE cameras SET sub_stream_url = $1 WHERE id = $2 AND sub_stream_url IS NULL")
                        .bind(sub)
                        .bind(camera.id)
                        .execute(&state.db_pool)
                        .await;
                    tracing::info!(camera_id = %camera.id, "Sub stream detected via ONVIF");
                }
            }
        }
    }
    ep
}

/// Background pass over all cameras at startup.
pub async fn discover_all(state: AppState) {
    tokio::time::sleep(Duration::from_secs(5)).await;
    let cameras = match state.camera_queries.list_cameras().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "PTZ discovery: cannot load cameras");
            return;
        }
    };
    for camera in cameras {
        discover(&state, &camera).await;
    }
}

fn error_response(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

async fn ptz_support(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let camera = state.camera_queries.get_camera(camera_id).await?;
    let ep = endpoint(&state, &camera).await;
    Ok(Json(serde_json::json!({ "supported": ep.is_some() })))
}

async fn ptz_command(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Json(cmd): Json<PtzCommand>,
) -> Response {
    if let Err(e) = cmd.validate() {
        return error_response(StatusCode::BAD_REQUEST, e);
    }
    let camera = match state.camera_queries.get_camera(camera_id).await {
        Ok(c) => c,
        Err(e) => return crate::error::ApiError::from(e).into_response(),
    };

    // Audit movements, not every release of the button.
    if cmd.action != "stop" {
        let audit = open_nvr_domain::entities::AuditLog::new(
            format!("ptz.{}", cmd.action),
            "camera".into(),
            Some(camera_id.to_string()),
        )
        .with_details(serde_json::json!({ "action": cmd.action, "speed": cmd.speed }));
        let _ = state.audit_repo.log(&audit).await;
    }

    let Some(ep) = endpoint(&state, &camera).await else {
        return error_response(StatusCode::UNPROCESSABLE_ENTITY, "Camera has no ONVIF PTZ service");
    };

    let speed = cmd.speed.unwrap_or(0.5).clamp(0.05, 1.0);
    let token = xml_escape(&ep.profile_token);
    let mv = |pan: f32, tilt: f32, zoom: Option<f32>| {
        let velocity = match zoom {
            Some(z) => format!(r#"<tt:Zoom x="{}"/>"#, z),
            None => format!(r#"<tt:PanTilt x="{}" y="{}"/>"#, pan, tilt),
        };
        format!(
            r#"<tptz:ContinuousMove><tptz:ProfileToken>{}</tptz:ProfileToken><tptz:Velocity>{}</tptz:Velocity></tptz:ContinuousMove>"#,
            token, velocity
        )
    };
    let body = match cmd.action.as_str() {
        "pan_left" => mv(-speed, 0.0, None),
        "pan_right" => mv(speed, 0.0, None),
        "tilt_up" => mv(0.0, speed, None),
        "tilt_down" => mv(0.0, -speed, None),
        "zoom_in" => mv(0.0, 0.0, Some(speed)),
        "zoom_out" => mv(0.0, 0.0, Some(-speed)),
        "stop" => format!(
            r#"<tptz:Stop><tptz:ProfileToken>{}</tptz:ProfileToken><tptz:PanTilt>true</tptz:PanTilt><tptz:Zoom>true</tptz:Zoom></tptz:Stop>"#,
            token
        ),
        "home" => format!(
            r#"<tptz:GotoHomePosition><tptz:ProfileToken>{}</tptz:ProfileToken></tptz:GotoHomePosition>"#,
            token
        ),
        _ => unreachable!("validated above"),
    };

    let creds = credentials(&camera);
    let mut result = send(&ep.ptz_url, &body, creds.as_ref()).await;

    // Plenty of cheap cameras have no home position but do have preset 1,
    // which their own app calls "home".
    if result.is_err() && cmd.action == "home" {
        let preset = format!(
            r#"<tptz:GotoPreset><tptz:ProfileToken>{}</tptz:ProfileToken><tptz:PresetToken>1</tptz:PresetToken></tptz:GotoPreset>"#,
            token
        );
        result = send(&ep.ptz_url, &preset, creds.as_ref()).await;
    }

    match result {
        Ok(_) => Json(serde_json::json!({
            "status": "ok",
            "camera_id": camera_id,
            "action": cmd.action,
            "speed": speed,
        }))
        .into_response(),
        Err(e) => {
            tracing::warn!(camera_id = %camera_id, action = %cmd.action, error = %e, "PTZ command failed");
            let message = if e.starts_with("camera returned HTTP 400") {
                format!("Camera does not support {}", cmd.action)
            } else {
                format!("PTZ failed: {}", e)
            };
            error_response(StatusCode::BAD_GATEWAY, message)
        }
    }
}

/// One PTZ call, retried once: these cameras routinely drop a request while
/// they are busy moving, and losing a `stop` would leave the camera panning.
async fn send(url: &str, body: &str, creds: Option<&(String, String)>) -> Result<String, String> {
    match soap(url, body, creds).await {
        Ok(v) => Ok(v),
        Err(e) if e.starts_with("camera unreachable") => {
            tokio::time::sleep(Duration::from_millis(300)).await;
            soap(url, body, creds).await
        }
        Err(e) => Err(e),
    }
}

async fn goto_preset(
    State(state): State<AppState>,
    Path(camera_id): Path<Uuid>,
    Json(cmd): Json<PresetCommand>,
) -> Response {
    if let Err(e) = cmd.validate() {
        return error_response(StatusCode::BAD_REQUEST, e);
    }
    let camera = match state.camera_queries.get_camera(camera_id).await {
        Ok(c) => c,
        Err(e) => return crate::error::ApiError::from(e).into_response(),
    };
    let Some(ep) = endpoint(&state, &camera).await else {
        return error_response(StatusCode::UNPROCESSABLE_ENTITY, "Camera has no ONVIF PTZ service");
    };

    let audit = open_nvr_domain::entities::AuditLog::new(
        "ptz.preset".into(),
        "camera".into(),
        Some(camera_id.to_string()),
    )
    .with_details(serde_json::json!({ "preset_id": cmd.preset_id }));
    let _ = state.audit_repo.log(&audit).await;

    let body = format!(
        r#"<tptz:GotoPreset><tptz:ProfileToken>{}</tptz:ProfileToken><tptz:PresetToken>{}</tptz:PresetToken></tptz:GotoPreset>"#,
        xml_escape(&ep.profile_token),
        cmd.preset_id
    );
    match send(&ep.ptz_url, &body, credentials(&camera).as_ref()).await {
        Ok(_) => Json(serde_json::json!({ "status": "ok", "camera_id": camera_id, "preset_id": cmd.preset_id }))
            .into_response(),
        Err(e) => error_response(StatusCode::BAD_GATEWAY, format!("PTZ failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_ptz_profile_and_xaddr() {
        let xml = r#"<trt:GetProfilesResponse><trt:Profiles fixed="false" token="stream0_0"><tt:Name>a</tt:Name><tt:PTZConfiguration token="Anv_ptz_0"/></trt:Profiles><trt:Profiles fixed="false" token="stream0_1"><tt:Name>b</tt:Name></trt:Profiles></trt:GetProfilesResponse>"#;
        assert_eq!(
            profiles(xml),
            vec![("stream0_0".to_string(), true), ("stream0_1".to_string(), false)]
        );
        let caps = r#"<tt:PTZ><tt:XAddr>http://10.0.0.2:8899/onvif/PTZ</tt:XAddr></tt:PTZ>"#;
        assert_eq!(element_text(caps, "XAddr"), Some("http://10.0.0.2:8899/onvif/PTZ"));
    }
}
