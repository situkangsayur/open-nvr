use open_nvr_domain::entities::{Camera, RecordingMode};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::go2rtc::Go2Rtc;

/// A stream that dies sooner than this after starting counts as a failed
/// start and pushes the next attempt further out.
const HEALTHY_AFTER: Duration = Duration::from_secs(60);
const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Manages per-camera ffmpeg processes that convert RTSP to HLS.
/// HLS segments are written to a local directory and served via HTTP.
/// Also manages recording processes that write MP4 segments to persistent storage.
pub struct HlsStreamManager {
    streams: Arc<RwLock<HashMap<Uuid, HlsStream>>>,
    cameras: Arc<RwLock<HashMap<Uuid, Camera>>>,
    /// Cameras whose last start failed, and when to try again.
    backoff: Arc<RwLock<HashMap<Uuid, Backoff>>>,
    hls_dir: PathBuf,
    rec_dir: PathBuf,
    log_dir: PathBuf,
    /// Read from the go2rtc loopback restream instead of the camera.
    via_go2rtc: AtomicBool,
}

struct HlsStream {
    hls_process: Child,
    rec_process: Option<Child>,
    /// UTC date of the directory the recorder writes into.
    rec_date: String,
    camera_name: String,
    started_at: Instant,
}

#[derive(Clone, Copy)]
struct Backoff {
    failures: u32,
    next_attempt: Instant,
}

impl HlsStreamManager {
    pub fn new(hls_dir: &str) -> Self {
        let path = PathBuf::from(hls_dir);
        std::fs::create_dir_all(&path).ok();

        // Recordings must outlive a reboot, so they default to a directory
        // under the working dir rather than /tmp. Override with RECORDINGS_DIR.
        let rec_path = std::env::var("RECORDINGS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./recordings"));
        if let Err(e) = std::fs::create_dir_all(&rec_path) {
            error!(dir = %rec_path.display(), error = %e, "Failed to create recordings directory");
        }

        let log_dir = std::env::var("FFMPEG_LOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp/opennvr-ffmpeg-logs"));
        std::fs::create_dir_all(&log_dir).ok();

        info!(
            hls_dir = %path.display(),
            recordings_dir = %rec_path.display(),
            "HLS/recording manager initialised"
        );

        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            cameras: Arc::new(RwLock::new(HashMap::new())),
            backoff: Arc::new(RwLock::new(HashMap::new())),
            hls_dir: path,
            rec_dir: rec_path,
            log_dir,
            via_go2rtc: AtomicBool::new(false),
        }
    }

    pub fn set_via_go2rtc(&self, on: bool) {
        self.via_go2rtc.store(on, Ordering::Relaxed);
    }

    /// Where ffmpeg should read this camera from.
    fn source_url(&self, camera: &Camera) -> String {
        if self.via_go2rtc.load(Ordering::Relaxed) {
            Go2Rtc::rtsp_url(camera.id)
        } else {
            camera.stream_url.clone()
        }
    }

    /// Open a truncating log file for an ffmpeg process, so a stream that
    /// fails to start leaves a diagnosable reason behind.
    fn ffmpeg_log(&self, camera_id: Uuid, kind: &str) -> std::process::Stdio {
        let path = self.log_dir.join(format!("{}-{}.log", camera_id, kind));
        match std::fs::File::create(&path) {
            Ok(file) => std::process::Stdio::from(file),
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Cannot open ffmpeg log, discarding stderr");
                std::process::Stdio::null()
            }
        }
    }

    /// Get the base recordings directory.
    pub fn recordings_dir(&self) -> &PathBuf {
        &self.rec_dir
    }

    /// Spawn the HLS ffmpeg process for a camera.
    fn spawn_hls_ffmpeg(
        stream_url: &str,
        seg_pattern: &str,
        playlist_path: &str,
        log: std::process::Stdio,
    ) -> Result<Child, String> {
        Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-rtsp_transport", "tcp",
                "-timeout", "5000000",    // RTSP timeout 5s (microseconds)
                // Some cameras send SPS/PPS late; without a generous probe
                // window ffmpeg gives up with "unspecified size" and writes
                // no output at all.
                "-analyzeduration", "10000000",
                "-probesize", "10000000",
                "-i", stream_url,
                "-c:v", "copy",
                // Audio is optional: cameras without an audio track must still
                // produce video, so map it with the "?" (optional) modifier.
                "-map", "0:v:0",
                "-map", "0:a:0?",
                "-c:a", "aac",
                "-ac", "1",
                "-ar", "44100",
                "-f", "hls",
                "-hls_time", "1",
                "-hls_list_size", "4",
                "-hls_flags", "delete_segments+append_list+omit_endlist",
                "-hls_segment_filename", seg_pattern,
                playlist_path,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(log)
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to start ffmpeg HLS: {}", e))
    }

    /// Spawn the recording ffmpeg process for a camera.
    fn spawn_rec_ffmpeg(
        stream_url: &str,
        rec_pattern: &str,
        log: std::process::Stdio,
    ) -> Option<Child> {
        match Command::new("ffmpeg")
            .args([
                "-nostdin",
                "-rtsp_transport", "tcp",
                "-timeout", "5000000",
                "-analyzeduration", "10000000",
                "-probesize", "10000000",
                "-i", stream_url,
                "-map", "0:v:0",
                "-c:v", "copy",
                "-an",
                "-f", "segment",
                "-segment_time", "300",
                "-segment_format", "mp4",
                // moov atom up front, so players start and seek without reading the whole file.
                "-segment_format_options", "movflags=+faststart",
                "-reset_timestamps", "1",
                "-strftime", "1",
                rec_pattern,
            ])
            .env("TZ", "UTC") // file names are parsed back as UTC
            .stdout(std::process::Stdio::null())
            .stderr(log)
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

    /// Start HLS streaming and recording for a camera (on-demand path used by
    /// the API; the reconcile loop covers everything else).
    pub async fn start(&self, camera: &Camera) -> Result<(), String> {
        let camera_id = camera.id;
        if self.streams.read().await.contains_key(&camera_id) {
            return Ok(());
        }
        self.cameras.write().await.insert(camera_id, camera.clone());
        // An explicit request skips any pending backoff.
        self.backoff.write().await.remove(&camera_id);
        self.start_streams(camera).await
    }

    fn today() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    /// Spawn the MP4 recording process for a camera into today's directory.
    /// Returns `None` if recording could not be started; live HLS is unaffected.
    fn start_recording(&self, camera: &Camera) -> Option<(Child, String)> {
        let camera_id = camera.id;
        let date = Self::today();
        let rec_camera_dir = self.rec_dir.join(camera_id.to_string()).join(&date);

        if let Err(e) = std::fs::create_dir_all(&rec_camera_dir) {
            error!(camera_id = %camera_id, dir = %rec_camera_dir.display(), error = %e, "Failed to create recording directory");
            return None;
        }

        let rec_pattern = rec_camera_dir.join("%Y%m%d_%H%M%S.mp4");
        let child = Self::spawn_rec_ffmpeg(
            &self.source_url(camera),
            rec_pattern.to_str()?,
            self.ffmpeg_log(camera_id, "rec"),
        )?;

        info!(camera_id = %camera_id, name = %camera.name, dir = %rec_camera_dir.display(), "Recording started");
        Some((child, date))
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
            &self.source_url(camera),
            seg_pattern.to_str().unwrap(),
            playlist_path.to_str().unwrap(),
            self.ffmpeg_log(camera_id, "hls"),
        )?;

        info!(camera_id = %camera_id, name = %camera.name, "HLS stream started");

        // Start recording. A recording failure is non-fatal: live HLS keeps running.
        let (rec_child, rec_date) = match self.start_recording(camera) {
            Some((c, d)) => (Some(c), d),
            None => (None, Self::today()),
        };

        let mut streams = self.streams.write().await;
        streams.insert(
            camera_id,
            HlsStream {
                hls_process: hls_child,
                rec_process: rec_child,
                rec_date,
                camera_name: camera.name.clone(),
                started_at: Instant::now(),
            },
        );

        Ok(())
    }

    /// Kill a camera's processes and forget its HLS files, but keep it known
    /// so the reconcile loop can start it again.
    async fn kill_streams(&self, camera_id: Uuid) {
        if let Some(mut stream) = self.streams.write().await.remove(&camera_id) {
            stream.hls_process.kill().await.ok();
            if let Some(mut rec) = stream.rec_process {
                rec.kill().await.ok();
            }
            info!(camera_id = %camera_id, name = %stream.camera_name, "HLS stream and recording stopped");
        }
        let cam_dir = self.hls_dir.join(camera_id.to_string());
        tokio::fs::remove_dir_all(&cam_dir).await.ok();
    }

    /// Stop HLS streaming and recording for a camera.
    pub async fn stop(&self, camera_id: Uuid) {
        self.kill_streams(camera_id).await;
        self.cameras.write().await.remove(&camera_id);
        self.backoff.write().await.remove(&camera_id);
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

    /// Get count of active streams.
    pub async fn active_count(&self) -> usize {
        self.streams.read().await.len()
    }

    /// Bring running processes in line with the camera table: every camera
    /// whose recording is not disabled gets HLS + recording, whatever status
    /// the health monitor last wrote (a camera that was offline at boot must
    /// still be picked up once it comes back). Removed/disabled cameras are
    /// stopped, and changed URLs restart. With `restart_all`, every running
    /// stream is restarted (the upstream restreamer was restarted).
    pub async fn reconcile(&self, cameras: &[Camera], restart_all: bool) {
        let wanted: HashMap<Uuid, &Camera> = cameras
            .iter()
            .filter(|c| c.recording_mode != RecordingMode::Disabled)
            .map(|c| (c.id, c))
            .collect();

        // Stop what is no longer wanted, restart what changed.
        let known: Vec<(Uuid, String)> = self
            .cameras
            .read()
            .await
            .iter()
            .map(|(id, c)| (*id, c.stream_url.clone()))
            .collect();
        for (id, url) in known {
            match wanted.get(&id) {
                None => self.stop(id).await,
                Some(cam) if cam.stream_url != url || restart_all => self.kill_streams(id).await,
                _ => {}
            }
        }

        {
            let mut known = self.cameras.write().await;
            for (id, cam) in &wanted {
                known.insert(*id, (*cam).clone());
            }
        }

        let now = Instant::now();
        let mut started = 0;
        for (id, cam) in &wanted {
            if self.streams.read().await.contains_key(id) {
                continue;
            }
            if let Some(b) = self.backoff.read().await.get(id) {
                if b.next_attempt > now {
                    continue;
                }
            }
            match self.start_streams(cam).await {
                Ok(()) => started += 1,
                Err(e) => {
                    error!(camera_id = %id, error = %e, "Failed to start HLS");
                    self.note_failure(*id).await;
                }
            }
        }
        if started > 0 {
            let active = self.active_count().await;
            info!(started, active, "Reconciled camera streams");
        }
    }

    async fn note_failure(&self, camera_id: Uuid) {
        let mut backoff = self.backoff.write().await;
        let entry = backoff.entry(camera_id).or_insert(Backoff {
            failures: 0,
            next_attempt: Instant::now(),
        });
        entry.failures += 1;
        let delay = Duration::from_secs(15 * 2u64.pow(entry.failures.min(5)))
            .min(MAX_BACKOFF);
        entry.next_attempt = Instant::now() + delay;
    }

    /// Watchdog: reap dead/hung ffmpeg processes. Dead HLS streams are
    /// dropped (the next reconcile restarts them, subject to backoff);
    /// recorders that died or crossed midnight are restarted in place.
    pub async fn check_and_restart(&self) {
        let mut rec_restart: Vec<Uuid> = Vec::new();
        let mut dropped = 0;
        let mut outcomes: Vec<(Uuid, bool)> = Vec::new();
        let today = Self::today();

        {
            let mut streams = self.streams.write().await;
            let mut to_remove = Vec::new();

            for (camera_id, stream) in streams.iter_mut() {
                let hls_dead = !matches!(stream.hls_process.try_wait(), Ok(None));

                // Check if m3u8 is stale (no update for > 30s = hung)
                let hls_hung = !hls_dead && {
                    let playlist = self.hls_dir.join(camera_id.to_string()).join("stream.m3u8");
                    match std::fs::metadata(&playlist).and_then(|m| m.modified()) {
                        Ok(modified) => modified.elapsed().map(|d| d.as_secs() > 30).unwrap_or(false),
                        // No playlist yet: give it the same 30 s to appear.
                        Err(_) => stream.started_at.elapsed() > Duration::from_secs(30),
                    }
                };

                if hls_dead || hls_hung {
                    let reason = if hls_dead { "exited" } else { "hung (stale m3u8)" };
                    warn!(camera_id = %camera_id, name = %stream.camera_name, reason = reason, "HLS ffmpeg dead/hung");
                    stream.hls_process.kill().await.ok();
                    if let Some(ref mut rec) = stream.rec_process {
                        rec.kill().await.ok();
                    }
                    let failed_fast = stream.started_at.elapsed() < HEALTHY_AFTER;
                    to_remove.push((*camera_id, failed_fast));
                    continue;
                }

                // HLS is fine, but the recording process may have died on its
                // own, or needs to roll over into the new day's directory.
                let rec_dead = match stream.rec_process {
                    Some(ref mut rec) => !matches!(rec.try_wait(), Ok(None)),
                    None => true,
                };
                if rec_dead || stream.rec_date != today {
                    if let Some(mut rec) = stream.rec_process.take() {
                        rec.kill().await.ok();
                    }
                    rec_restart.push(*camera_id);
                }
            }

            for (id, failed_fast) in to_remove {
                streams.remove(&id);
                dropped += 1;
                let cam_dir = self.hls_dir.join(id.to_string());
                tokio::fs::remove_dir_all(&cam_dir).await.ok();
                outcomes.push((id, failed_fast));
            }
        }

        for (id, failed_fast) in outcomes {
            if failed_fast {
                self.note_failure(id).await;
            } else {
                self.backoff.write().await.remove(&id);
            }
        }

        let mut rec_restarted = 0;
        for camera_id in rec_restart {
            let camera = self.cameras.read().await.get(&camera_id).cloned();
            let Some(camera) = camera else { continue };

            if let Some((child, date)) = self.start_recording(&camera) {
                let mut streams = self.streams.write().await;
                if let Some(stream) = streams.get_mut(&camera_id) {
                    stream.rec_process = Some(child);
                    stream.rec_date = date;
                    rec_restarted += 1;
                }
            }
        }

        if dropped > 0 || rec_restarted > 0 {
            info!(streams_dropped = dropped, recordings_restarted = rec_restarted, "Watchdog pass complete");
        }
    }
}
