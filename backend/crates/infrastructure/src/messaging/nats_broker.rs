use open_nvr_domain::entities::{AuditLog, NetworkEvent};
use tracing::{info, warn};

/// NATS message broker for Open-NVR event pub/sub.
///
/// Subjects:
/// - `opennvr.camera.{id}.status` — Camera online/offline/error status changes
/// - `opennvr.camera.{id}.frame` — New frame available (metadata only, not the actual frame)
/// - `opennvr.detection.{camera_id}` — Detection events (motion, human, etc.)
/// - `opennvr.audit` — Audit log entries
/// - `opennvr.network.security` — Network security events
/// - `opennvr.recording.{camera_id}` — Recording segment completed
pub struct NatsBroker {
    client: async_nats::Client,
}

impl NatsBroker {
    pub async fn connect(url: &str) -> Result<Self, String> {
        let client = async_nats::connect(url)
            .await
            .map_err(|e| format!("Failed to connect to NATS: {}", e))?;

        info!(url = url, "Connected to NATS broker");
        Ok(Self { client })
    }

    /// Publish camera status change
    pub async fn publish_camera_status(&self, camera_id: &str, status: &str) {
        let subject = format!("opennvr.camera.{}.status", camera_id);
        let payload = serde_json::json!({
            "camera_id": camera_id,
            "status": status,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Err(e) = self
            .client
            .publish(subject.clone(), payload.to_string().into())
            .await
        {
            warn!(subject = %subject, error = %e, "Failed to publish camera status");
        }
    }

    /// Publish detection event
    pub async fn publish_detection(&self, camera_id: &str, event_type: &str, confidence: f32) {
        let subject = format!("opennvr.detection.{}", camera_id);
        let payload = serde_json::json!({
            "camera_id": camera_id,
            "event_type": event_type,
            "confidence": confidence,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Err(e) = self
            .client
            .publish(subject, payload.to_string().into())
            .await
        {
            warn!(error = %e, "Failed to publish detection event");
        }
    }

    /// Publish audit log entry
    pub async fn publish_audit(&self, log: &AuditLog) {
        let payload = serde_json::json!({
            "id": log.id,
            "user_id": log.user_id,
            "action": log.action,
            "resource_type": log.resource_type,
            "resource_id": log.resource_id,
            "timestamp": log.created_at.to_rfc3339(),
        });

        if let Err(e) = self
            .client
            .publish("opennvr.audit".to_string(), payload.to_string().into())
            .await
        {
            warn!(error = %e, "Failed to publish audit event");
        }
    }

    /// Publish network security event
    pub async fn publish_network_event(&self, event: &NetworkEvent) {
        let payload = serde_json::json!({
            "id": event.id,
            "event_type": event.event_type.to_string(),
            "severity": event.severity.to_string(),
            "source_ip": event.source_ip.map(|ip| ip.to_string()),
            "timestamp": event.created_at.to_rfc3339(),
        });

        if let Err(e) = self
            .client
            .publish(
                "opennvr.network.security".to_string(),
                payload.to_string().into(),
            )
            .await
        {
            warn!(error = %e, "Failed to publish network event");
        }
    }

    /// Publish recording segment complete
    pub async fn publish_recording_segment(
        &self,
        camera_id: &str,
        recording_id: &str,
        segment_seq: i32,
    ) {
        let subject = format!("opennvr.recording.{}", camera_id);
        let payload = serde_json::json!({
            "camera_id": camera_id,
            "recording_id": recording_id,
            "segment_seq": segment_seq,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Err(e) = self
            .client
            .publish(subject, payload.to_string().into())
            .await
        {
            warn!(error = %e, "Failed to publish recording segment");
        }
    }

    /// Subscribe to a subject and return a subscriber
    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber, String> {
        self.client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| format!("Failed to subscribe to {}: {}", subject, e))
    }

    /// Get the underlying client for advanced usage
    pub fn client(&self) -> &async_nats::Client {
        &self.client
    }
}

/// Optional NATS broker wrapper — if NATS is not configured, operations are no-ops.
pub struct OptionalNats {
    broker: Option<NatsBroker>,
}

impl OptionalNats {
    pub async fn from_env() -> Self {
        let nats_url = std::env::var("NATS_URL").ok();

        let broker = if let Some(url) = nats_url {
            match NatsBroker::connect(&url).await {
                Ok(b) => {
                    info!("NATS broker connected");
                    Some(b)
                }
                Err(e) => {
                    warn!(error = %e, "NATS not available, running without message broker");
                    None
                }
            }
        } else {
            info!("NATS_URL not set, running without message broker");
            None
        };

        Self { broker }
    }

    pub async fn publish_camera_status(&self, camera_id: &str, status: &str) {
        if let Some(ref b) = self.broker {
            b.publish_camera_status(camera_id, status).await;
        }
    }

    pub async fn publish_detection(&self, camera_id: &str, event_type: &str, confidence: f32) {
        if let Some(ref b) = self.broker {
            b.publish_detection(camera_id, event_type, confidence).await;
        }
    }

    pub async fn publish_audit(&self, log: &AuditLog) {
        if let Some(ref b) = self.broker {
            b.publish_audit(log).await;
        }
    }

    pub async fn publish_network_event(&self, event: &NetworkEvent) {
        if let Some(ref b) = self.broker {
            b.publish_network_event(event).await;
        }
    }

    pub async fn publish_recording_segment(
        &self,
        camera_id: &str,
        recording_id: &str,
        segment_seq: i32,
    ) {
        if let Some(ref b) = self.broker {
            b.publish_recording_segment(camera_id, recording_id, segment_seq)
                .await;
        }
    }

    pub fn is_connected(&self) -> bool {
        self.broker.is_some()
    }
}
