/// Calculate estimated storage needs.
pub struct StorageCalculator;

impl StorageCalculator {
    /// Calculate storage in bytes for given parameters.
    /// bitrate_mbps: camera bitrate in Mbps
    /// retention_days: number of days to retain
    /// num_cameras: number of cameras
    /// motion_only_ratio: 0.0-1.0, portion of time with motion (1.0 = continuous)
    pub fn estimate_bytes(
        bitrate_mbps: f64,
        retention_days: u32,
        num_cameras: u32,
        motion_only_ratio: f64,
    ) -> u64 {
        let bytes_per_sec = bitrate_mbps * 1_000_000.0 / 8.0;
        let secs_per_day = 86400.0;
        let total = bytes_per_sec
            * secs_per_day
            * retention_days as f64
            * num_cameras as f64
            * motion_only_ratio;
        total as u64
    }

    /// Format bytes to human-readable string.
    pub fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if bytes >= TB {
            format!("{:.1} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.1} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.1} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_estimate_4cam_7days() {
        let bytes = StorageCalculator::estimate_bytes(4.0, 7, 4, 1.0);
        // 4 Mbps = 500,000 bytes/sec * 86400 sec/day * 7 days * 4 cameras = ~1,126 GB
        let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        assert!(gb > 1100.0 && gb < 1150.0, "Expected ~1126 GB, got {:.1} GB", gb);
    }

    #[test]
    fn test_motion_only_reduces_storage() {
        let continuous = StorageCalculator::estimate_bytes(4.0, 30, 8, 1.0);
        let motion = StorageCalculator::estimate_bytes(4.0, 30, 8, 0.3);
        assert!(motion < continuous);
        assert!((motion as f64 / continuous as f64 - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(StorageCalculator::format_bytes(500), "500 B");
        assert_eq!(StorageCalculator::format_bytes(1024 * 1024 * 512), "512.0 MB");
    }
}
