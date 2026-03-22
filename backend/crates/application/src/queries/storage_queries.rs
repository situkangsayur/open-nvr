use open_nvr_domain::services::StorageCalculator;
use crate::dto::*;

pub struct StorageQueryService;

impl StorageQueryService {
    pub fn estimate(req: &StorageEstimateRequest) -> StorageEstimate {
        let ratio = req.motion_ratio.unwrap_or(1.0);
        let bytes = StorageCalculator::estimate_bytes(
            req.bitrate_mbps,
            req.retention_days,
            req.cameras,
            ratio,
        );
        StorageEstimate {
            total_bytes: bytes,
            formatted: StorageCalculator::format_bytes(bytes),
            cameras: req.cameras,
            bitrate_mbps: req.bitrate_mbps,
            retention_days: req.retention_days,
            motion_ratio: ratio,
        }
    }
}
