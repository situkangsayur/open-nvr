use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    pub fn new(
        action: String,
        resource_type: String,
        resource_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            user_email: None,
            action,
            resource_type,
            resource_id,
            details: serde_json::json!({}),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_user(mut self, user_id: String, user_email: Option<String>) -> Self {
        self.user_id = Some(user_id);
        self.user_email = user_email;
        self
    }

    pub fn with_request_info(mut self, ip: Option<IpAddr>, user_agent: Option<String>) -> Self {
        self.ip_address = ip;
        self.user_agent = user_agent;
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkEventType {
    UnauthorizedAccess,
    PortScan,
    BruteForce,
    UnknownDevice,
    ProtocolViolation,
    ConnectionAnomaly,
}

impl std::fmt::Display for NetworkEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnauthorizedAccess => write!(f, "unauthorized_access"),
            Self::PortScan => write!(f, "port_scan"),
            Self::BruteForce => write!(f, "brute_force"),
            Self::UnknownDevice => write!(f, "unknown_device"),
            Self::ProtocolViolation => write!(f, "protocol_violation"),
            Self::ConnectionAnomaly => write!(f, "connection_anomaly"),
        }
    }
}

impl std::str::FromStr for NetworkEventType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unauthorized_access" => Ok(Self::UnauthorizedAccess),
            "port_scan" => Ok(Self::PortScan),
            "brute_force" => Ok(Self::BruteForce),
            "unknown_device" => Ok(Self::UnknownDevice),
            "protocol_violation" => Ok(Self::ProtocolViolation),
            "connection_anomaly" => Ok(Self::ConnectionAnomaly),
            _ => Err(format!("Unknown network event type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "info" => Ok(Self::Info),
            "warning" => Ok(Self::Warning),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub id: Uuid,
    pub event_type: NetworkEventType,
    pub source_ip: Option<IpAddr>,
    pub source_mac: Option<String>,
    pub target_resource: Option<String>,
    pub severity: Severity,
    pub details: serde_json::Value,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl NetworkEvent {
    pub fn new(event_type: NetworkEventType, severity: Severity) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            source_ip: None,
            source_mac: None,
            target_resource: None,
            severity,
            details: serde_json::json!({}),
            resolved: false,
            resolved_at: None,
            resolved_by: None,
            created_at: Utc::now(),
        }
    }

    pub fn resolve(&mut self, resolved_by: String) {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
        self.resolved_by = Some(resolved_by);
    }
}
