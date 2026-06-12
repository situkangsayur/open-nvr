use open_nvr_domain::entities::Camera;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Manages per-camera ffmpeg processes that convert RTSP to HLS.
/// HLS segments are written to a local directory and served via HTTP.
/// Also manages recording processes that write MP4 segments to persistent storage.
pub struct HlsStreamManager {
    streams: Arc<RwLock<HashMap<Uuid, HlsStream>>>,
    cameras: Arc<RwLock<HashMap<Uuid, Camera>>>,
    hls_dir: PathBuf,
    rec_dir: PathBuf,
}

struct HlsStream {
    hls_process: Child,
    rec_process: Option<Child>,
    camera_name: String,
}

impl HlsStreamManager {
    pub fn new(hls_dir: &str) -> Self {
        let path = PathBuf::from(hls_dir);
        std::fs::create_dir_all(&path).ok();
        let rec_path = PathBuf::from("/tmp/opennvr-recordings");
        std::fs::create_dir_all(&rec_path).ok();
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            cameras: Arc::new(RwLock::new(HashMap::new())),
            hls_dir: path,
            rec_dir: rec_path,
        }
    }

    /// Get the base recordings directory.
    pub fn recordings_dir(&self) -> &PathBuf {
        &self.rec_dir
    }

    /// Spawn the HLS ffmpeg process for a camera.
    fn spawn_hls_ffmpeg(stream_url: &str, seg_pattern: &str, playlist_path: &str) -> Result<Child, String> {
        Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-rtsp_transport", "tcp",
                "-timeout", "5000000",    // RTSP timeout 5s (microseconds)
                "-i", stream_url,
                "-c:v", "copy",
                "-c:a", "aac",
                "-ac", "1",
                "-ar", "44100",
                "-f", "hls",
                "-hls_time", "2",
                "-hls_list_size", "5",
                "-hls_flags", "delete_segments+append_list",
                "-hls_segment_filename", seg_pattern,
                playlist_path,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to start ffmpeg HLS: {}", e))
    }

    /// Spawn the recording ffmpeg process for a camera.
    fn spawn_rec_ffmpeg(stream_url: &str, rec_pattern: &str) -> Option<Child> {
        match Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-rtsp_transport", "tcp",
                "-timeout", "5000000",
                "-i", stream_url,
                "-c:v", "copy",
                "-an",
                "-f", "segment",
                "-segment_time", "300",
                "-segment_format", "mp4",
                "-reset_timestamps", "1",
                "-strftime", "1",
                rec_pattern,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(child) => Some(child),
            Err(e) => {
                warn!(error = %e, "Failed to start recording ffmpeg");
                None
            }
        }
    }

    /// Start HLS streaming and recording for a camera.
    pub async fn start(&self, camera: &Camera) -> Result<(), String> {
        let camera_id = camera.id;

        // Check if already running
        {
            let streams = self.streams.read().await;
            if streams.contains_key(&camera_id) {
                return Ok(());
            }
        }

        // Store camera info for watchdog restarts
        {
            let mut cameras = self.cameras.write().await;
            cameras.insert(camera_id, camera.clone());
        }

        self.start_streams(camera).await
    }

    /// Internal: start ffmpeg processes for a camera (used by start and watchdog).
    async fn start_streams(&self, camera: &Camera) -> Result<(), String> {
        let camera_id = camera.id;

        // Create camera-specific HLS directory
        let cam_dir = self.hls_dir.join(camera_id.to_string());
        std::fs::create_dir_all(&cam_dir).map_err(|e| e.to_string())?;

        let playlist_path = cam_dir.join("stream.m3u8");
        let seg_pattern = cam_dir.join("seg_%03d.ts");

        let hls_child = Self::spawn_hls_ffmpeg(
            &camera.stream_url,
            seg_pattern.to_str().unwrap(),
            playlist_path.to_str().unwrap(),
        )?;

        info!(camera_id = %camera_id, name = %camera.name, "HLS stream started");

        // Start recording
        let rec_camera_dir = self.rec_dir.join(camera_id.to_string())
            .join(chrono::Utc::now().format("%Y-%m-%d").to_string());
        std::fs::create_dir_all(&rec_camera_dir).map_err(|e| e.to_string())?;

        let rec_pattern = rec_camera_dir.join("%Y%m%d_%H%M%S.mp4");
        let rec_child = Self::spawn_rec_ffmpeg(
            &camera.stream_url,
            rec_pattern.to_str().unwrap(),
        );

        if rec_child.is_some() {
            info!(camera_id = %camera_id, name = %camera.name, dir = %rec_camera_dir.display(), "Recording started");
        }

        let mut streams = self.streams.write().await;
        streams.insert(
            camera_id,
            HlsStream {
                hls_process: hls_child,
                rec_process: rec_child,
                camera_name: camera.name.clone(),
            },
        );

        Ok(())
    }

    /// Stop HLS streaming and recording for a camera.
    pub async fn stop(&self, camera_id: Uuid) {
        let mut streams = self.streams.write().await;
        if let Some(mut stream) = streams.remove(&camera_id) {
            stream.hls_process.kill().await.ok();
            if let Some(mut rec) = stream.rec_process {
                rec.kill().await.ok();
            }
            info!(camera_id = %camera_id, name = %stream.camera_name, "HLS stream and recording stopped");
        }

        // Remove from camera list
        {
            let mut cameras = self.cameras.write().await;
            cameras.remove(&camera_id);
        }

        // Clean up HLS files (not recordings - those are persistent)
        let cam_dir = self.hls_dir.join(camera_id.to_string());
        tokio::fs::remove_dir_all(&cam_dir).await.ok();
    }

    /// Check if a camera has an active HLS stream.
    pub async fn is_streaming(&self, camera_id: &Uuid) -> bool {
        let streams = self.streams.read().await;
        streams.contains_key(camera_id)
    }

    /// Get the HLS playlist path for a camera.
    pub fn playlist_path(&self, camera_id: &Uuid) -> PathBuf {
        self.hls_dir.join(camera_id.to_string()).join("stream.m3u8")
    }

    /// Get the HLS directory for a camera (for serving segments).
    pub fn camera_dir(&self, camera_id: &Uuid) -> PathBuf {
        self.hls_dir.join(camera_id.to_string())
    }

    /// Start HLS for all online cameras.
    pub async fn start_all(&self, cameras: &[Camera]) {
        for camera in cameras {
            if camera.status == open_nvr_domain::entities::CameraStatus::Online {
                if let Err(e) = self.start(camera).await {
                    error!(camera_id = %camera.id, error = %e, "Failed to start HLS");
                }
            }
        }
    }

    /// Get count of active streams.
    pub async fn active_count(&self) -> usize {
        self.streams.read().await.len()
    }

    /// Watchdog: check all streams and restart dead/hung ffmpeg processes.
    /// Should be called periodically (e.g. every 30 seconds).
    pub async fn check_and_restart(&self) {
        let mut dead_cameras: Vec<Camera> = Vec::new();

        // Check for dead/hung processes
        {
            let mut streams = self.streams.write().await;
            let mut to_remove = Vec::new();

            for (camera_id, stream) in streams.iter_mut() {
                let hls_dead = match stream.hls_process.try_wait() {
                    Ok(Some(_exit)) => true,  // Process exited
                    Ok(None) => false,         // Still running
                    Err(_) => true,            // Error checking = assume dead
                };

                // Check if m3u8 is stale (no update for > 30s = hung)
                let hls_hung = if !hls_dead {
                    let playlist = self.hls_dir.join(camera_id.to_string()).join("stream.m3u8");
                    match std::fs::metadata(&playlist) {
                        Ok(meta) => {
                            if let Ok(modified) = meta.modified() {
                                modified.elapsed().map(|d| d.as_secs() > 30).unwrap_or(false)
                            } else {
                                false
                            }
                        }
                        Err(_) => false, // File doesn't exist yet, give it time
                    }
                } else {
                    false
                };

                if hls_dead || hls_hung {
                    let reason = if hls_dead { "exited" } else { "hung (stale m3u8)" };
                    warn!(camera_id = %camera_id, name = %stream.camera_name, reason = reason, "HLS ffmpeg detected as dead/hung, will restart");

                    // Kill hung process
                    if hls_hung {
                        stream.hls_process.kill().await.ok();
                    }

                    // Kill recording process too
                    if let Some(ref mut rec) = stream.rec_process {
                        rec.kill().await.ok();
                    }

                    to_remove.push(*camera_id);
                }
            }

            // Remove dead streams and collect cameras for restart
            let cameras = self.cameras.read().await;
            for id in to_remove {
                streams.remove(&id);
                // Clean up stale HLS files
                let cam_dir = self.hls_dir.join(id.to_string());
                tokio::fs::remove_dir_all(&cam_dir).await.ok();

                if let Some(cam) = cameras.get(&id) {
                    dead_cameras.push(cam.clone());
                }
            }
        }

        // Restart dead streams
        for camera in &dead_cameras {
            info!(camera_id = %camera.id, name = %camera.name, "Restarting HLS stream");
            if let Err(e) = self.start_streams(camera).await {
                error!(camera_id = %camera.id, error = %e, "Failed to restart HLS stream");
            }
        }

        if !dead_cameras.is_empty() {
            info!(restarted = dead_cameras.len(), "Watchdog restart complete");
        }
    }
}
