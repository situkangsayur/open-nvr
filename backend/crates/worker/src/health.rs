use open_nvr_domain::entities::CameraStatus;
use open_nvr_domain::ports::CameraRepository;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn};

/// Background task that monitors camera connection health.
pub async fn camera_health_monitor(camera_repo: Arc<dyn CameraRepository>) {
    let mut ticker = interval(Duration::from_secs(30));

    loop {
        ticker.tick().await;

        match camera_repo.find_by_status(CameraStatus::Online).await {
            Ok(cameras) => {
                info!(count = cameras.len(), "Health check: online cameras");
            }
            Err(e) => {
                warn!(error = %e, "Health check failed to query cameras");
            }
        }
    }
}
