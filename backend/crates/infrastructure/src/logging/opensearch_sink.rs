use open_nvr_domain::entities::{AuditLog, NetworkEvent};
use tracing::info;

/// Sends audit logs and network events to OpenSearch for analysis.
/// This is optional — if OpenSearch is not configured, logs only go to PostgreSQL.
pub struct OpenSearchSink {
    client: reqwest::Client,
    base_url: String,
    audit_index: String,
    network_index: String,
}

impl OpenSearchSink {
    pub fn new(base_url: &str, audit_index: &str, network_index: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            audit_index: audit_index.to_string(),
            network_index: network_index.to_string(),
        }
    }

    pub async fn index_audit_log(&self, log: &AuditLog) -> Result<(), String> {
        let url = format!("{}/{}/_doc/{}", self.base_url, self.audit_index, log.id);
        let body = serde_json::json!({
            "id": log.id,
            "user_id": log.user_id,
            "user_email": log.user_email,
            "action": log.action,
            "resource_type": log.resource_type,
            "resource_id": log.resource_id,
            "details": log.details,
            "ip_address": log.ip_address.map(|ip| ip.to_string()),
            "user_agent": log.user_agent,
            "@timestamp": log.created_at.to_rfc3339(),
        });

        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to index audit log: {}", e))?;

        Ok(())
    }

    pub async fn index_network_event(&self, event: &NetworkEvent) -> Result<(), String> {
        let url = format!("{}/{}/_doc/{}", self.base_url, self.network_index, event.id);
        let body = serde_json::json!({
            "id": event.id,
            "event_type": event.event_type.to_string(),
            "source_ip": event.source_ip.map(|ip| ip.to_string()),
            "source_mac": event.source_mac,
            "target_resource": event.target_resource,
            "severity": event.severity.to_string(),
            "details": event.details,
            "resolved": event.resolved,
            "@timestamp": event.created_at.to_rfc3339(),
        });

        self.client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to index network event: {}", e))?;

        Ok(())
    }

    pub async fn ensure_indices(&self) -> Result<(), String> {
        for index in [&self.audit_index, &self.network_index] {
            let url = format!("{}/{}", self.base_url, index);
            let resp = self.client.head(&url).send().await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    info!(index = %index, "OpenSearch index exists");
                }
                _ => {
                    let mapping = serde_json::json!({
                        "mappings": {
                            "properties": {
                                "@timestamp": { "type": "date" },
                                "user_id": { "type": "keyword" },
                                "action": { "type": "keyword" },
                                "resource_type": { "type": "keyword" },
                                "event_type": { "type": "keyword" },
                                "severity": { "type": "keyword" },
                                "source_ip": { "type": "ip" },
                                "details": { "type": "object", "enabled": false },
                            }
                        }
                    });
                    self.client
                        .put(&url)
                        .json(&mapping)
                        .send()
                        .await
                        .map_err(|e| format!("Failed to create index {}: {}", index, e))?;
                    info!(index = %index, "OpenSearch index created");
                }
            }
        }
        Ok(())
    }
}
