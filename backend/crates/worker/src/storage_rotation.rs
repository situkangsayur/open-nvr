use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Default share of the recordings filesystem Open-NVR is allowed to fill.
pub const DEFAULT_MAX_DISK_PERCENT: f64 = 80.0;
/// Default gap below the cap that a rotation pass frees up, so we do not run
/// a delete pass on every tick once the disk sits right at the limit.
pub const DEFAULT_RECLAIM_MARGIN_PERCENT: f64 = 5.0;
const DEFAULT_INTERVAL_SECS: u64 = 300;
/// Segments younger than this are assumed to still be open by ffmpeg.
const MIN_AGE_BEFORE_DELETE: Duration = Duration::from_secs(120);

/// Configuration for the recording rotation worker, read from the environment.
#[derive(Debug, Clone)]
pub struct RotationConfig {
    pub recordings_dir: PathBuf,
    /// Delete oldest segments once the filesystem is fuller than this.
    pub max_disk_percent: f64,
    /// Free down to `max_disk_percent - reclaim_margin_percent` in one pass.
    pub reclaim_margin_percent: f64,
    /// Hard age cap in days. `0` disables age-based deletion (disk cap only).
    pub retention_days: u32,
    pub interval: Duration,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            recordings_dir: PathBuf::from("./recordings"),
            max_disk_percent: DEFAULT_MAX_DISK_PERCENT,
            reclaim_margin_percent: DEFAULT_RECLAIM_MARGIN_PERCENT,
            retention_days: 0,
            interval: Duration::from_secs(DEFAULT_INTERVAL_SECS),
        }
    }
}

impl RotationConfig {
    /// Build the config from environment variables, falling back to defaults.
    ///
    /// - `RECORDINGS_DIR`
    /// - `RECORDINGS_MAX_DISK_PERCENT` (default 80)
    /// - `RECORDINGS_RECLAIM_MARGIN_PERCENT` (default 5)
    /// - `RECORDINGS_RETENTION_DAYS` (default 0 = disk cap only)
    /// - `RECORDINGS_ROTATION_INTERVAL_SECS` (default 300)
    pub fn from_env() -> Self {
        let default = Self::default();

        let recordings_dir = std::env::var("RECORDINGS_DIR")
            .map(PathBuf::from)
            .unwrap_or(default.recordings_dir);

        let max_disk_percent = env_f64("RECORDINGS_MAX_DISK_PERCENT")
            .unwrap_or(default.max_disk_percent)
            .clamp(1.0, 99.0);

        let reclaim_margin_percent = env_f64("RECORDINGS_RECLAIM_MARGIN_PERCENT")
            .unwrap_or(default.reclaim_margin_percent)
            .clamp(0.0, max_disk_percent - 1.0);

        let retention_days = env_u64("RECORDINGS_RETENTION_DAYS")
            .map(|v| v as u32)
            .unwrap_or(default.retention_days);

        let interval = env_u64("RECORDINGS_ROTATION_INTERVAL_SECS")
            .filter(|s| *s > 0)
            .map(Duration::from_secs)
            .unwrap_or(default.interval);

        Self {
            recordings_dir,
            max_disk_percent,
            reclaim_margin_percent,
            retention_days,
            interval,
        }
    }

    /// Usage level a rotation pass frees down to once the cap is exceeded.
    pub fn target_percent(&self) -> f64 {
        (self.max_disk_percent - self.reclaim_margin_percent).max(1.0)
    }
}

fn env_f64(key: &str) -> Option<f64> {
    std::env::var(key).ok()?.trim().parse::<f64>().ok()
}

fn env_u64(key: &str) -> Option<u64> {
    std::env::var(key).ok()?.trim().parse::<u64>().ok()
}

/// Filesystem capacity numbers for the partition holding the recordings.
#[derive(Debug, Clone, Copy)]
pub struct DiskUsage {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
}

impl DiskUsage {
    pub fn used_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        self.used_bytes as f64 * 100.0 / self.total_bytes as f64
    }

    /// Bytes that must be freed to bring usage down to `target_percent`.
    pub fn bytes_over(&self, target_percent: f64) -> u64 {
        let allowed = (self.total_bytes as f64 * target_percent / 100.0) as u64;
        self.used_bytes.saturating_sub(allowed)
    }
}

/// Read filesystem usage for the partition that contains `path`.
///
/// Uses the same accounting as `df`: capacity excludes root-reserved blocks,
/// so the percentage here matches what an operator sees on the box.
pub fn disk_usage(path: &Path) -> Result<DiskUsage, String> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())
        .map_err(|e| format!("invalid path: {}", e))?;

    // SAFETY: `stat` is fully initialised by a successful statvfs call, and
    // `c_path` is a valid NUL-terminated string that outlives the call.
    let stat = unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return Err(format!(
                "statvfs({}) failed: {}",
                path.display(),
                std::io::Error::last_os_error()
            ));
        }
        stat
    };

    let block = stat.f_frsize as u64;
    let available = stat.f_bavail as u64 * block;
    let used = (stat.f_blocks as u64).saturating_sub(stat.f_bfree as u64) * block;
    // `df` computes Use% against used + available, which excludes the
    // root-reserved blocks. Match that so our numbers agree with the box.
    let capacity = used + available;

    Ok(DiskUsage {
        total_bytes: capacity,
        used_bytes: used,
        available_bytes: available,
    })
}

#[derive(Debug, Clone)]
struct Segment {
    path: PathBuf,
    size_bytes: u64,
    modified: SystemTime,
}

/// Collect every recorded MP4 segment under `dir`, oldest first.
fn collect_segments(dir: &Path) -> Vec<Segment> {
    let mut out = Vec::new();
    collect_segments_into(dir, &mut out);
    out.sort_by_key(|s| s.modified);
    out
}

fn collect_segments_into(dir: &Path, out: &mut Vec<Segment>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            debug!(dir = %dir.display(), error = %e, "Cannot read recordings directory");
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.is_dir() {
            collect_segments_into(&path, out);
        } else if meta.is_file() && path.extension().is_some_and(|e| e == "mp4") {
            let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            out.push(Segment {
                path,
                size_bytes: meta.len(),
                modified,
            });
        }
    }
}

/// True when the segment is old enough that ffmpeg is no longer writing it.
fn is_closed(segment: &Segment) -> bool {
    segment
        .modified
        .elapsed()
        .map(|age| age >= MIN_AGE_BEFORE_DELETE)
        .unwrap_or(false)
}

/// Delete date/camera directories left empty after their segments were removed.
fn prune_empty_dirs(root: &Path) {
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        prune_empty_dirs(&path);
        if std::fs::read_dir(&path).map(|mut d| d.next().is_none()).unwrap_or(false) {
            if std::fs::remove_dir(&path).is_ok() {
                debug!(dir = %path.display(), "Removed empty recording directory");
            }
        }
    }
}

/// Delete segments older than `retention_days`. Returns (files, bytes) removed.
fn enforce_age_limit(segments: &mut Vec<Segment>, retention_days: u32) -> (usize, u64) {
    if retention_days == 0 {
        return (0, 0);
    }

    let max_age = Duration::from_secs(retention_days as u64 * 86_400);
    let mut files = 0;
    let mut bytes = 0;

    segments.retain(|segment| {
        let too_old = segment
            .modified
            .elapsed()
            .map(|age| age > max_age)
            .unwrap_or(false);

        if !too_old || !is_closed(segment) {
            return true;
        }

        match std::fs::remove_file(&segment.path) {
            Ok(()) => {
                files += 1;
                bytes += segment.size_bytes;
                false
            }
            Err(e) => {
                warn!(path = %segment.path.display(), error = %e, "Failed to delete expired segment");
                true
            }
        }
    });

    (files, bytes)
}

/// Delete oldest segments until `bytes_to_free` has been reclaimed.
/// Returns (files, bytes) removed.
fn enforce_disk_limit(segments: &[Segment], bytes_to_free: u64) -> (usize, u64) {
    let mut files = 0;
    let mut freed = 0u64;

    for segment in segments {
        if freed >= bytes_to_free {
            break;
        }
        // Never delete the segment ffmpeg is currently writing.
        if !is_closed(segment) {
            continue;
        }

        match std::fs::remove_file(&segment.path) {
            Ok(()) => {
                files += 1;
                freed += segment.size_bytes;
                debug!(path = %segment.path.display(), size = segment.size_bytes, "Rotated out old segment");
            }
            Err(e) => {
                warn!(path = %segment.path.display(), error = %e, "Failed to delete segment");
            }
        }
    }

    (files, freed)
}

/// Run a single rotation pass: age limit first, then the disk cap.
pub fn run_once(config: &RotationConfig) -> Result<(), String> {
    let dir = &config.recordings_dir;
    if !dir.exists() {
        debug!(dir = %dir.display(), "Recordings directory does not exist yet, skipping rotation");
        return Ok(());
    }

    let mut segments = collect_segments(dir);
    let (aged_files, aged_bytes) = enforce_age_limit(&mut segments, config.retention_days);
    if aged_files > 0 {
        info!(
            files = aged_files,
            freed_mb = aged_bytes / 1_048_576,
            retention_days = config.retention_days,
            "Deleted segments past retention age"
        );
    }

    let usage = disk_usage(dir)?;
    let used_percent = usage.used_percent();

    if used_percent <= config.max_disk_percent {
        debug!(
            used_percent = format!("{:.1}", used_percent),
            max_percent = config.max_disk_percent,
            "Disk below cap, no rotation needed"
        );
        return Ok(());
    }

    let bytes_to_free = usage.bytes_over(config.target_percent());
    warn!(
        used_percent = format!("{:.1}", used_percent),
        max_percent = config.max_disk_percent,
        target_percent = format!("{:.1}", config.target_percent()),
        to_free_mb = bytes_to_free / 1_048_576,
        "Disk over cap, rotating oldest recordings"
    );

    let (files, freed) = enforce_disk_limit(&segments, bytes_to_free);
    prune_empty_dirs(dir);

    let after = disk_usage(dir).map(|u| u.used_percent()).unwrap_or(used_percent);
    if files == 0 {
        error!(
            used_percent = format!("{:.1}", after),
            "Disk over cap but no deletable recordings found - free space manually"
        );
    } else {
        info!(
            files = files,
            freed_mb = freed / 1_048_576,
            used_percent = format!("{:.1}", after),
            "Rotation complete"
        );
    }

    Ok(())
}

/// Background worker that keeps recordings within the configured disk budget.
pub async fn storage_rotation_worker(config: RotationConfig) {
    info!(
        dir = %config.recordings_dir.display(),
        max_disk_percent = config.max_disk_percent,
        target_percent = format!("{:.1}", config.target_percent()),
        retention_days = config.retention_days,
        interval_secs = config.interval.as_secs(),
        "Storage rotation worker started"
    );

    let mut ticker = interval(config.interval);
    loop {
        ticker.tick().await;

        let cfg = config.clone();
        // Directory walks and unlinks are blocking syscalls; keep them off the
        // async runtime's worker threads.
        let result = tokio::task::spawn_blocking(move || run_once(&cfg)).await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => error!(error = %e, "Storage rotation pass failed"),
            Err(e) => error!(error = %e, "Storage rotation task panicked"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn used_percent_and_overage() {
        let usage = DiskUsage {
            total_bytes: 1000,
            used_bytes: 900,
            available_bytes: 100,
        };
        assert_eq!(usage.used_percent(), 90.0);
        // Target 75% => allowed 750, used 900 => free 150.
        assert_eq!(usage.bytes_over(75.0), 150);
        // Already below target => nothing to free.
        assert_eq!(usage.bytes_over(95.0), 0);
    }

    #[test]
    fn target_percent_sits_below_cap() {
        let config = RotationConfig {
            max_disk_percent: 80.0,
            reclaim_margin_percent: 5.0,
            ..RotationConfig::default()
        };
        assert_eq!(config.target_percent(), 75.0);
    }

    #[test]
    fn disk_usage_reads_real_filesystem() {
        let usage = disk_usage(Path::new(".")).expect("statvfs on cwd should succeed");
        assert!(usage.total_bytes > 0);
        assert!(usage.used_percent() >= 0.0 && usage.used_percent() <= 100.0);
    }
}
