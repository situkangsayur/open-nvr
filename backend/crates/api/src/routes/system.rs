use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::error::ApiResult;
use crate::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/system/info", get(system_info))
        .route("/system/capacity", get(capacity_analysis))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct SystemInfo {
    hostname: String,
    os: String,
    kernel: String,
    cpu_model: String,
    cpu_cores: u32,
    cpu_threads: u32,
    ram_total_gb: f64,
    ram_available_gb: f64,
    ram_used_percent: f64,
    disk_total_gb: f64,
    disk_available_gb: f64,
    disk_used_percent: f64,
    gpu: Option<String>,
    load_avg: [f64; 3],
    uptime_hours: f64,
    ffmpeg_available: bool,
}

#[derive(Debug, Serialize)]
struct CapacityAnalysis {
    system: SystemInfo,
    capacity: CameraCapacity,
    recommendations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CameraCapacity {
    max_cameras_1080p_continuous: u32,
    max_cameras_720p_continuous: u32,
    max_cameras_1080p_motion: u32,
    estimated_recording_days_1080p: u32,
    estimated_recording_days_720p: u32,
    cpu_per_camera_percent: f64,
    ram_per_camera_mb: u32,
    disk_per_camera_per_day_gb: f64,
    network_bandwidth_per_camera_mbps: f64,
}

async fn system_info() -> Json<SystemInfo> {
    Json(get_system_info().await)
}

async fn capacity_analysis(
    State(state): State<AppState>,
) -> ApiResult<Json<CapacityAnalysis>> {
    let info = get_system_info().await;
    let capacity = calculate_capacity(&info);
    let recommendations = generate_recommendations(&info, &capacity);

    // Get current camera count
    let camera_count = state.camera_queries.list_cameras().await
        .map(|c| c.len() as u32)
        .unwrap_or(0);

    let mut recs = recommendations;
    recs.insert(0, format!("Currently {} cameras configured", camera_count));

    Ok(Json(CapacityAnalysis {
        system: info,
        capacity,
        recommendations: recs,
    }))
}

async fn get_system_info() -> SystemInfo {
    let hostname = read_file_line("/etc/hostname").await.unwrap_or_else(|| "unknown".into());
    let os = match read_cmd("lsb_release", &["-ds"]).await {
        Some(val) => val,
        None => {
            // Fallback: try to read PRETTY_NAME from /etc/os-release
            read_file("/etc/os-release").await
                .and_then(|content| {
                    content.lines()
                        .find(|l| l.starts_with("PRETTY_NAME"))
                        .and_then(|l| l.split('=').nth(1))
                        .map(|s| s.trim_matches('"').to_string())
                })
                .unwrap_or_default()
        }
    };
    let kernel = read_cmd("uname", &["-r"]).await.unwrap_or_default();

    // CPU info
    let cpuinfo = read_file("/proc/cpuinfo").await.unwrap_or_default();
    let cpu_model = cpuinfo.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown CPU".into());
    let cpu_threads = cpuinfo.lines()
        .filter(|l| l.starts_with("processor"))
        .count() as u32;
    let cpu_cores = cpuinfo.lines()
        .find(|l| l.starts_with("cpu cores"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(cpu_threads);

    // Memory
    let meminfo = read_file("/proc/meminfo").await.unwrap_or_default();
    let mem_total_kb = parse_meminfo_kb(&meminfo, "MemTotal");
    let mem_available_kb = parse_meminfo_kb(&meminfo, "MemAvailable");
    let ram_total_gb = mem_total_kb as f64 / 1024.0 / 1024.0;
    let ram_available_gb = mem_available_kb as f64 / 1024.0 / 1024.0;
    let ram_used_percent = if mem_total_kb > 0 {
        ((mem_total_kb - mem_available_kb) as f64 / mem_total_kb as f64) * 100.0
    } else { 0.0 };

    // Disk
    let df_output = read_cmd("df", &["-B1", "/"]).await.unwrap_or_default();
    let (disk_total, disk_available) = parse_df(&df_output);
    let disk_total_gb = disk_total as f64 / 1024.0 / 1024.0 / 1024.0;
    let disk_available_gb = disk_available as f64 / 1024.0 / 1024.0 / 1024.0;
    let disk_used_percent = if disk_total > 0 {
        ((disk_total - disk_available) as f64 / disk_total as f64) * 100.0
    } else { 0.0 };

    // GPU
    let gpu = read_cmd("nvidia-smi", &["--query-gpu=name", "--format=csv,noheader"]).await;

    // Load average
    let loadavg = read_file("/proc/loadavg").await.unwrap_or_default();
    let loads: Vec<f64> = loadavg.split_whitespace()
        .take(3)
        .filter_map(|s| s.parse().ok())
        .collect();
    let load_avg = [
        loads.first().copied().unwrap_or(0.0),
        loads.get(1).copied().unwrap_or(0.0),
        loads.get(2).copied().unwrap_or(0.0),
    ];

    // Uptime
    let uptime_str = read_file("/proc/uptime").await.unwrap_or_default();
    let uptime_secs: f64 = uptime_str.split_whitespace().next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    // ffmpeg
    let ffmpeg_available = read_cmd("which", &["ffmpeg"]).await.is_some();

    SystemInfo {
        hostname: hostname.trim().to_string(),
        os: os.trim().to_string(),
        kernel: kernel.trim().to_string(),
        cpu_model,
        cpu_cores,
        cpu_threads,
        ram_total_gb: (ram_total_gb * 10.0).round() / 10.0,
        ram_available_gb: (ram_available_gb * 10.0).round() / 10.0,
        ram_used_percent: (ram_used_percent * 10.0).round() / 10.0,
        disk_total_gb: (disk_total_gb * 10.0).round() / 10.0,
        disk_available_gb: (disk_available_gb * 10.0).round() / 10.0,
        disk_used_percent: (disk_used_percent * 10.0).round() / 10.0,
        gpu,
        load_avg,
        uptime_hours: (uptime_secs / 3600.0 * 10.0).round() / 10.0,
        ffmpeg_available,
    }
}

fn calculate_capacity(info: &SystemInfo) -> CameraCapacity {
    // Estimates based on typical NVR resource usage
    // Each 1080p camera RTSP stream: ~5% CPU, ~100MB RAM, ~4Mbps bandwidth
    // Each 720p camera: ~3% CPU, ~60MB RAM, ~2Mbps
    let cpu_available_percent = (100.0 * info.cpu_threads as f64) - (info.load_avg[0] / info.cpu_threads as f64 * 100.0);
    let ram_available_mb = info.ram_available_gb * 1024.0;

    let cpu_per_1080p = 5.0;
    let ram_per_1080p = 100.0;
    let bandwidth_per_1080p = 4.0; // Mbps
    let disk_per_1080p_day_gb = 4.0 * 86400.0 / 8.0 / 1024.0; // ~42 GB/day

    let cpu_per_720p = 3.0;
    let ram_per_720p = 60.0;
    let disk_per_720p_day_gb = 2.0 * 86400.0 / 8.0 / 1024.0; // ~21 GB/day

    let max_by_cpu_1080p = (cpu_available_percent / cpu_per_1080p) as u32;
    let max_by_ram_1080p = (ram_available_mb / ram_per_1080p) as u32;
    let max_1080p = max_by_cpu_1080p.min(max_by_ram_1080p).min(64);

    let max_by_cpu_720p = (cpu_available_percent / cpu_per_720p) as u32;
    let max_by_ram_720p = (ram_available_mb / ram_per_720p) as u32;
    let max_720p = max_by_cpu_720p.min(max_by_ram_720p).min(128);

    let recording_days_1080p = if max_1080p > 0 {
        (info.disk_available_gb / (disk_per_1080p_day_gb * max_1080p as f64)) as u32
    } else { 0 };

    let recording_days_720p = if max_720p > 0 {
        (info.disk_available_gb / (disk_per_720p_day_gb * max_720p as f64)) as u32
    } else { 0 };

    CameraCapacity {
        max_cameras_1080p_continuous: max_1080p,
        max_cameras_720p_continuous: max_720p,
        max_cameras_1080p_motion: max_1080p * 3, // Motion-only ~30% duty cycle
        estimated_recording_days_1080p: recording_days_1080p,
        estimated_recording_days_720p: recording_days_720p,
        cpu_per_camera_percent: cpu_per_1080p,
        ram_per_camera_mb: ram_per_1080p as u32,
        disk_per_camera_per_day_gb: (disk_per_1080p_day_gb * 10.0).round() / 10.0,
        network_bandwidth_per_camera_mbps: bandwidth_per_1080p,
    }
}

fn generate_recommendations(info: &SystemInfo, cap: &CameraCapacity) -> Vec<String> {
    let mut recs = Vec::new();

    if info.ram_available_gb < 2.0 {
        recs.push("Low RAM available. Consider closing other applications or adding more RAM.".into());
    }
    if info.disk_used_percent > 80.0 {
        recs.push(format!("Disk usage is {:.0}%. Consider adding storage or enabling retention policies.", info.disk_used_percent));
    }
    if info.load_avg[0] > info.cpu_threads as f64 * 0.8 {
        recs.push("CPU load is high. Camera recording may be affected.".into());
    }
    if info.gpu.is_some() {
        recs.push("GPU detected. Hardware-accelerated decoding can be enabled for better performance.".into());
    }
    if !info.ffmpeg_available {
        recs.push("ffmpeg not installed. Snapshots and video export will not work. Install with: apt install ffmpeg".into());
    }

    recs.push(format!("Max {} cameras at 1080p continuous, or {} at 720p", cap.max_cameras_1080p_continuous, cap.max_cameras_720p_continuous));
    recs.push(format!("Each 1080p camera uses ~{:.1} GB/day of storage", cap.disk_per_camera_per_day_gb));

    if cap.estimated_recording_days_1080p > 0 {
        recs.push(format!("Available disk can store ~{} days of recording at max camera load", cap.estimated_recording_days_1080p));
    }

    recs
}

async fn read_file(path: &str) -> Option<String> {
    tokio::fs::read_to_string(path).await.ok()
}

async fn read_file_line(path: &str) -> Option<String> {
    read_file(path).await.map(|s| s.lines().next().unwrap_or("").to_string())
}

async fn read_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    tokio::process::Command::new(cmd)
        .args(args)
        .output()
        .await
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn parse_meminfo_kb(content: &str, key: &str) -> u64 {
    content.lines()
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn parse_df(output: &str) -> (u64, u64) {
    output.lines().nth(1)
        .map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            let total = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0u64);
            let available = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0u64);
            (total, available)
        })
        .unwrap_or((0, 0))
}
