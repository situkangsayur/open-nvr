use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use crate::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub ip: IpAddr,
    pub mac: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub name: Option<String>,
    pub protocols: Vec<String>,
    pub rtsp_url: Option<String>,
    pub onvif_url: Option<String>,
    pub http_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRequest {
    pub subnets: Vec<String>,
    pub timeout_secs: u64,
    pub include_onvif: bool,
    pub include_mdns: bool,
    pub include_arp: bool,
}

impl Default for ScanRequest {
    fn default() -> Self {
        Self {
            subnets: Vec::new(),
            timeout_secs: 30,
            include_onvif: true,
            include_mdns: true,
            include_arp: true,
        }
    }
}

#[async_trait]
pub trait DeviceDiscovery: Send + Sync {
    async fn scan(&self, request: &ScanRequest) -> Result<Vec<DiscoveredDevice>, DomainError>;
    async fn scan_progress(&self) -> Result<f32, DomainError>;
}
