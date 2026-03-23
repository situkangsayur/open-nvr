use axum::routing::post;
use axum::{Json, Router};
use open_nvr_application::dto::ScanNetworkRequest;
use open_nvr_domain::ports::{DeviceDiscovery, DiscoveredDevice, ScanRequest};
use open_nvr_infrastructure::discovery::scanner::NetworkScanner;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/discovery/scan", post(scan_network))
        .with_state(state)
}

async fn scan_network(
    Json(req): Json<ScanNetworkRequest>,
) -> ApiResult<Json<Vec<DiscoveredDevice>>> {
    req.validate()
        .map_err(|e| open_nvr_domain::errors::DomainError::Validation(e))?;

    let scanner = NetworkScanner::new();

    let scan_request = ScanRequest {
        subnets: req.subnets.unwrap_or_else(|| vec!["192.168.1.0/24".into()]),
        timeout_secs: req.timeout_secs.unwrap_or(30),
        include_onvif: true,
        include_mdns: true,
        include_arp: true,
    };

    let devices = scanner.scan(&scan_request).await?;

    Ok(Json(devices))
}
