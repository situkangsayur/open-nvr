use open_nvr_domain::entities::CameraStatus;
use open_nvr_domain::ports::CameraRepository;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::time::{interval, timeout, Duration};
use tracing::{debug, info, warn};

/// Background task that monitors camera connection health.
/// Periodically pings each camera's RTSP/HTTP port and updates status.
pub async fn camera_health_monitor(camera_repo: Arc<dyn CameraRepository>) {
    // Wait 10 seconds for startup
    tokio::time::sleep(Duration::from_secs(10)).await;

    let mut ticker = interval(Duration::from_secs(60)); // Check every 60 seconds

    loop {
        ticker.tick().await;

        let cameras = match camera_repo.find_all().await {
            Ok(cams) => cams,
            Err(e) => {
                warn!(error = %e, "Health check failed to query cameras");
                continue;
            }
        };

        if cameras.is_empty() {
            continue;
        }

        let mut online = 0u32;
        let mut offline = 0u32;

        for camera in &cameras {
            // Extract host:port from stream URL
            let host_port = extract_host_port(&camera.stream_url);

            let is_alive = if let Some((host, port)) = host_port {
                let addr = format!("{}:{}", host, port);
                match timeout(Duration::from_secs(3), TcpStream::connect(&addr)).await {
                    Ok(Ok(_)) => true,
                    _ => false,
                }
            } else {
                false
            };

            let new_status = if is_alive {
                online += 1;
                CameraStatus::Online
            } else {
                offline += 1;
                CameraStatus::Offline
            };

            // Only update if status changed
            if (is_alive && camera.status != CameraStatus::Online) ||
               (!is_alive && camera.status != CameraStatus::Offline) {
                let _ = camera_repo.update_status(camera.id, new_status).await;
                debug!(
                    camera_id = %camera.id,
                    name = %camera.name,
                    alive = is_alive,
                    "Camera status updated"
                );
            }
        }

        info!(total = cameras.len(), online = online, offline = offline, "Health check complete");
    }
}

fn extract_host_port(url: &str) -> Option<(String, u16)> {
    let without_scheme = url.split("://").nth(1)?;
    let without_auth = without_scheme.split('@').last()?;
    let host_port_path = without_auth.split('/').next()?;

    let parts: Vec<&str> = host_port_path.split(':').collect();
    let host = parts[0].to_string();
    let port = if parts.len() > 1 {
        parts[1].parse().unwrap_or(554)
    } else {
        554 // Default RTSP port
    };

    Some((host, port))
}
