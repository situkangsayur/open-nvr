use open_nvr_domain::entities::*;
use open_nvr_domain::ports::*;
use open_nvr_infrastructure::detection::motion::MotionDetector;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::recording::LiveFrame;

/// Detection worker that processes live frames for motion detection.
/// Samples keyframes and runs motion detection + zone filtering.
pub async fn detection_worker(
    camera_id: Uuid,
    camera_name: String,
    zones: Vec<DetectionZone>,
    mut live_rx: broadcast::Receiver<LiveFrame>,
    event_repo: Arc<dyn DetectionEventRepository>,
    frame_width: u32,
    frame_height: u32,
) {
    info!(camera_id = %camera_id, name = %camera_name, "Starting detection worker");

    let mut detector = MotionDetector::new(frame_width, frame_height);
    let mut frame_count = 0u64;
    let sample_interval = 30; // Process every 30th frame (~1/sec at 30fps)

    loop {
        match live_rx.recv().await {
            Ok(frame) => {
                if frame.camera_id != camera_id || frame.is_audio {
                    continue;
                }

                frame_count += 1;
                if frame_count % sample_interval != 0 {
                    continue;
                }

                // Run motion detection
                let regions = detector.detect(&frame.data);

                for region in &regions {
                    // Check if motion falls within any enabled detection zone
                    let matching_zones: Vec<&DetectionZone> = if zones.is_empty() {
                        // No zones configured = detect everywhere
                        vec![]
                    } else {
                        zones
                            .iter()
                            .filter(|z| z.enabled)
                            .filter(|z| z.detection_types.contains(&DetectionEventType::Motion))
                            .filter(|z| detector.region_in_zone(region, &z.polygon))
                            .collect()
                    };

                    // Create detection event
                    let zone_id = matching_zones.first().map(|z| z.id);

                    // Skip if zones are configured but no zone matches
                    if !zones.is_empty() && matching_zones.is_empty() {
                        continue;
                    }

                    let event = DetectionEvent {
                        id: Uuid::new_v4(),
                        camera_id,
                        zone_id,
                        event_type: DetectionEventType::Motion,
                        confidence: Some(region.intensity),
                        bounding_box: Some(region.to_bounding_box(frame_width, frame_height)),
                        thumbnail_key: None,
                        metadata: serde_json::json!({
                            "region_x": region.x,
                            "region_y": region.y,
                            "region_width": region.width,
                            "region_height": region.height,
                        }),
                        occurred_at: chrono::Utc::now(),
                        created_at: chrono::Utc::now(),
                    };

                    debug!(
                        camera_id = %camera_id,
                        intensity = region.intensity,
                        "Motion detected"
                    );

                    if let Err(e) = event_repo.create(&event).await {
                        error!(error = %e, "Failed to save detection event");
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                debug!(skipped = n, "Detection worker lagging");
            }
            Err(broadcast::error::RecvError::Closed) => {
                info!(camera_id = %camera_id, "Detection worker stopping - channel closed");
                break;
            }
        }
    }
}
