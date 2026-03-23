use open_nvr_domain::entities::*;
use open_nvr_domain::ports::*;
use open_nvr_infrastructure::crypto::credentials::CredentialEncryptor as InfraEncryptor;
use open_nvr_infrastructure::protocols::create_stream_ingester;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::recording::{camera_recording_task, LiveFrame};

/// Manages per-camera background tasks (recording, detection, reconnection).
/// When a camera is added/enabled, tasks are spawned. When removed/disabled, tasks are stopped.
pub struct CameraManager {
    camera_repo: Arc<dyn CameraRepository>,
    recording_repo: Arc<dyn RecordingRepository>,
    segment_repo: Arc<dyn RecordingSegmentRepository>,
    _event_repo: Arc<dyn DetectionEventRepository>,
    _zone_repo: Arc<dyn DetectionZoneRepository>,
    storage: Option<Arc<dyn ObjectStorage>>,
    encryptor: Option<InfraEncryptor>,
    live_tx: broadcast::Sender<LiveFrame>,
    tasks: Arc<RwLock<HashMap<Uuid, CameraTask>>>,
}

struct CameraTask {
    recording_handle: JoinHandle<()>,
    camera_name: String,
}

impl CameraManager {
    pub fn new(
        camera_repo: Arc<dyn CameraRepository>,
        recording_repo: Arc<dyn RecordingRepository>,
        segment_repo: Arc<dyn RecordingSegmentRepository>,
        event_repo: Arc<dyn DetectionEventRepository>,
        zone_repo: Arc<dyn DetectionZoneRepository>,
        storage: Option<Arc<dyn ObjectStorage>>,
        encryptor: Option<InfraEncryptor>,
        live_tx: broadcast::Sender<LiveFrame>,
    ) -> Self {
        Self {
            camera_repo,
            recording_repo,
            segment_repo,
            _event_repo: event_repo,
            _zone_repo: zone_repo,
            storage,
            encryptor,
            live_tx,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start tasks for all cameras that have recording enabled.
    pub async fn start_all(&self) {
        let cameras = match self.camera_repo.find_all().await {
            Ok(cams) => cams,
            Err(e) => {
                error!(error = %e, "Failed to load cameras for manager");
                return;
            }
        };

        for camera in cameras {
            if camera.recording_mode != RecordingMode::Disabled {
                self.start_camera(&camera).await;
            }
        }
    }

    /// Start recording + detection tasks for a single camera.
    pub async fn start_camera(&self, camera: &Camera) {
        let camera_id = camera.id;

        // Don't start if already running
        {
            let tasks = self.tasks.read().await;
            if tasks.contains_key(&camera_id) {
                warn!(camera_id = %camera_id, "Camera tasks already running");
                return;
            }
        }

        info!(camera_id = %camera_id, name = %camera.name, protocol = %camera.protocol_type, "Starting camera tasks");

        // Decrypt credentials if available
        let (username, password) = if let Some(ref encrypted) = camera.credentials_encrypted {
            if let Some(ref enc) = self.encryptor {
                match enc.decrypt(encrypted) {
                    Ok(creds) => {
                        let parts: Vec<&str> = creds.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            (Some(parts[0].to_string()), Some(parts[1].to_string()))
                        } else {
                            (None, None)
                        }
                    }
                    Err(e) => {
                        warn!(camera_id = %camera_id, error = %e, "Failed to decrypt credentials");
                        (None, None)
                    }
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Create protocol adapter
        let ingester = match create_stream_ingester(
            &camera.protocol_type.to_string(),
            &camera.stream_url,
            username.as_deref(),
            password.as_deref(),
        ) {
            Ok(ing) => ing,
            Err(e) => {
                error!(camera_id = %camera_id, error = %e, "Failed to create stream ingester");
                return;
            }
        };

        // Clone what we need for the spawned task
        let camera_clone = camera.clone();
        let camera_repo = self.camera_repo.clone();
        let recording_repo = self.recording_repo.clone();
        let segment_repo = self.segment_repo.clone();
        let storage = self.storage.clone();
        let live_tx = self.live_tx.clone();

        // Use a dummy storage if none configured
        let storage_for_recording: Arc<dyn ObjectStorage> = match storage {
            Some(s) => s,
            None => Arc::new(NullStorage),
        };

        let recording_handle = tokio::spawn(async move {
            camera_recording_task(
                camera_clone,
                ingester,
                storage_for_recording,
                camera_repo,
                recording_repo,
                segment_repo,
                live_tx,
            )
            .await;
        });

        let mut tasks = self.tasks.write().await;
        tasks.insert(
            camera_id,
            CameraTask {
                recording_handle,
                camera_name: camera.name.clone(),
            },
        );

        info!(camera_id = %camera_id, "Camera tasks started");
    }

    /// Stop tasks for a camera.
    pub async fn stop_camera(&self, camera_id: Uuid) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.remove(&camera_id) {
            info!(camera_id = %camera_id, name = %task.camera_name, "Stopping camera tasks");
            task.recording_handle.abort();
        }
    }

    /// Get count of running camera tasks.
    pub async fn running_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }
}

/// Null storage that discards all writes (used when MinIO is not configured)
struct NullStorage;

#[async_trait::async_trait]
impl ObjectStorage for NullStorage {
    async fn put_object(
        &self,
        key: &str,
        _data: &[u8],
        _content_type: &str,
    ) -> Result<(), open_nvr_domain::errors::DomainError> {
        tracing::debug!(key = key, "NullStorage: discarding object (MinIO not configured)");
        Ok(())
    }
    async fn get_object(&self, _key: &str) -> Result<Vec<u8>, open_nvr_domain::errors::DomainError> {
        Err(open_nvr_domain::errors::DomainError::Storage(
            "Storage not configured".into(),
        ))
    }
    async fn delete_object(&self, _key: &str) -> Result<(), open_nvr_domain::errors::DomainError> {
        Ok(())
    }
    async fn presigned_url(
        &self,
        _key: &str,
        _expiry_secs: u64,
    ) -> Result<String, open_nvr_domain::errors::DomainError> {
        Err(open_nvr_domain::errors::DomainError::Storage(
            "Storage not configured".into(),
        ))
    }
    async fn list_objects(
        &self,
        _prefix: &str,
    ) -> Result<Vec<String>, open_nvr_domain::errors::DomainError> {
        Ok(Vec::new())
    }
    async fn get_total_size(
        &self,
        _prefix: &str,
    ) -> Result<u64, open_nvr_domain::errors::DomainError> {
        Ok(0)
    }
}
