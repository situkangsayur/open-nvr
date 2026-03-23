use open_nvr_domain::ports::*;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, error};
use chrono::Utc;

/// Background worker that enforces retention policies.
/// Runs periodically and deletes recordings older than the policy allows.
pub async fn retention_cleanup_worker(
    retention_repo: Arc<dyn RetentionPolicyRepository>,
    _recording_repo: Arc<dyn RecordingRepository>,
    _segment_repo: Arc<dyn RecordingSegmentRepository>,
    _storage: Arc<dyn ObjectStorage>,
) {
    let mut ticker = interval(Duration::from_secs(3600)); // Run every hour

    loop {
        ticker.tick().await;
        info!("Running retention cleanup");

        let policies = match retention_repo.find_all().await {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "Failed to fetch retention policies");
                continue;
            }
        };

        for policy in &policies {
            if !policy.enabled {
                continue;
            }

            let cutoff = Utc::now() - chrono::Duration::days(policy.retention_days as i64);
            info!(
                policy = %policy.name,
                retention_days = policy.retention_days,
                cutoff = %cutoff,
                "Checking retention policy"
            );

            // TODO: Query recordings before cutoff and delete them + their segments from MinIO
            // This requires adding a find_before_date method to RecordingRepository
            // For now, just log what would be cleaned up
        }
    }
}
