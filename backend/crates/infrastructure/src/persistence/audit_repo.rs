use async_trait::async_trait;
use chrono::{DateTime, Utc};
use open_nvr_domain::entities::*;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::{AuditRepository, NetworkEventRepository};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn audit_log_from_row(r: sqlx::postgres::PgRow) -> AuditLog {
    let ip_str: Option<String> = r.get("ip_address");
    AuditLog {
        id: r.get("id"),
        user_id: r.get("user_id"),
        user_email: r.get("user_email"),
        action: r.get("action"),
        resource_type: r.get("resource_type"),
        resource_id: r.get("resource_id"),
        details: r.get::<Option<serde_json::Value>, _>("details").unwrap_or_default(),
        ip_address: ip_str.and_then(|s| s.parse().ok()),
        user_agent: r.get("user_agent"),
        created_at: r.get("created_at"),
    }
}

#[async_trait]
impl AuditRepository for PgAuditRepository {
    async fn log(&self, entry: &AuditLog) -> Result<(), DomainError> {
        let ip_str = entry.ip_address.map(|ip| ip.to_string());
        sqlx::query(
            r#"INSERT INTO audit_logs (id, user_id, user_email, action, resource_type, resource_id,
                details, ip_address, user_agent, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8::inet, $9, $10)"#,
        )
        .bind(entry.id)
        .bind(&entry.user_id)
        .bind(&entry.user_email)
        .bind(&entry.action)
        .bind(&entry.resource_type)
        .bind(&entry.resource_id)
        .bind(&entry.details)
        .bind(&ip_str)
        .bind(&entry.user_agent)
        .bind(entry.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_by_user(&self, user_id: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<AuditLog>, DomainError> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, user_email, action, resource_type, resource_id,
                      details, ip_address::text, user_agent, created_at
               FROM audit_logs WHERE user_id = $1 AND created_at BETWEEN $2 AND $3
               ORDER BY created_at DESC"#,
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(audit_log_from_row).collect())
    }

    async fn find_by_resource(&self, resource_type: &str, resource_id: &str) -> Result<Vec<AuditLog>, DomainError> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, user_email, action, resource_type, resource_id,
                      details, ip_address::text, user_agent, created_at
               FROM audit_logs WHERE resource_type = $1 AND resource_id = $2
               ORDER BY created_at DESC"#,
        )
        .bind(resource_type)
        .bind(resource_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(audit_log_from_row).collect())
    }

    async fn find_recent(&self, limit: i64) -> Result<Vec<AuditLog>, DomainError> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, user_email, action, resource_type, resource_id,
                      details, ip_address::text, user_agent, created_at
               FROM audit_logs ORDER BY created_at DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(audit_log_from_row).collect())
    }
}

pub struct PgNetworkEventRepository {
    pool: PgPool,
}

impl PgNetworkEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn network_event_from_row(r: sqlx::postgres::PgRow) -> NetworkEvent {
    let ip_str: Option<String> = r.get("source_ip");
    NetworkEvent {
        id: r.get("id"),
        event_type: r.get::<String, _>("event_type").parse().unwrap_or(NetworkEventType::ConnectionAnomaly),
        source_ip: ip_str.and_then(|s| s.parse().ok()),
        source_mac: r.get("source_mac"),
        target_resource: r.get("target_resource"),
        severity: r.get::<String, _>("severity").parse().unwrap_or(Severity::Info),
        details: r.get::<Option<serde_json::Value>, _>("details").unwrap_or_default(),
        resolved: r.get("resolved"),
        resolved_at: r.get("resolved_at"),
        resolved_by: r.get("resolved_by"),
        created_at: r.get("created_at"),
    }
}

#[async_trait]
impl NetworkEventRepository for PgNetworkEventRepository {
    async fn create(&self, event: &NetworkEvent) -> Result<(), DomainError> {
        let source_ip_str = event.source_ip.map(|ip| ip.to_string());
        sqlx::query(
            r#"INSERT INTO network_events (id, event_type, source_ip, source_mac, target_resource,
                severity, details, resolved, created_at)
               VALUES ($1, $2, $3::inet, $4, $5, $6, $7, $8, $9)"#,
        )
        .bind(event.id)
        .bind(event.event_type.to_string())
        .bind(&source_ip_str)
        .bind(&event.source_mac)
        .bind(&event.target_resource)
        .bind(event.severity.to_string())
        .bind(&event.details)
        .bind(event.resolved)
        .bind(event.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_unresolved(&self) -> Result<Vec<NetworkEvent>, DomainError> {
        let rows = sqlx::query(
            r#"SELECT id, event_type, source_ip::text, source_mac, target_resource,
                      severity, details, resolved, resolved_at, resolved_by, created_at
               FROM network_events WHERE resolved = false
               ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(network_event_from_row).collect())
    }

    async fn find_by_severity(&self, severity: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<NetworkEvent>, DomainError> {
        let rows = sqlx::query(
            r#"SELECT id, event_type, source_ip::text, source_mac, target_resource,
                      severity, details, resolved, resolved_at, resolved_by, created_at
               FROM network_events WHERE severity = $1 AND created_at BETWEEN $2 AND $3
               ORDER BY created_at DESC"#,
        )
        .bind(severity)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(network_event_from_row).collect())
    }

    async fn resolve(&self, id: Uuid, resolved_by: &str) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE network_events SET resolved = true, resolved_at = now(), resolved_by = $2 WHERE id = $1",
        )
        .bind(id)
        .bind(resolved_by)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}
