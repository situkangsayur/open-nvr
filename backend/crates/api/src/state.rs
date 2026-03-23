use open_nvr_application::commands::*;
use open_nvr_application::queries::*;
use open_nvr_domain::ports::*;
use open_nvr_infrastructure::persistence::*;
use open_nvr_worker::manager::CameraManager;
use open_nvr_worker::recording::LiveFrame;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub camera_commands: Arc<CameraCommandService>,
    pub camera_queries: Arc<CameraQueryService>,
    pub group_commands: Arc<GroupCommandService>,
    pub recording_queries: Arc<RecordingQueryService>,
    pub timeline_queries: Arc<TimelineQueryService>,
    pub audit_repo: Arc<dyn AuditRepository>,
    pub network_event_repo: Arc<dyn NetworkEventRepository>,
    pub zone_repo: Arc<dyn DetectionZoneRepository>,
    pub event_repo: Arc<dyn DetectionEventRepository>,
    pub layout_repo: Arc<dyn GridLayoutRepository>,
    pub retention_repo: Arc<dyn RetentionPolicyRepository>,
    pub camera_manager: Option<Arc<CameraManager>>,
    pub db_pool: PgPool,
    pub live_tx: broadcast::Sender<LiveFrame>,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let camera_repo = Arc::new(PgCameraRepository::new(pool.clone()));
        let group_repo = Arc::new(PgCameraGroupRepository::new(pool.clone()));
        let audit_repo = Arc::new(PgAuditRepository::new(pool.clone()));
        let network_event_repo = Arc::new(PgNetworkEventRepository::new(pool.clone()));
        let recording_repo = Arc::new(PgRecordingRepository::new(pool.clone()));
        let segment_repo = Arc::new(PgRecordingSegmentRepository::new(pool.clone()));
        let event_repo = Arc::new(PgDetectionEventRepository::new(pool.clone()));
        let zone_repo = Arc::new(PgDetectionZoneRepository::new(pool.clone()));
        let layout_repo = Arc::new(PgGridLayoutRepository::new(pool.clone()));
        let retention_repo = Arc::new(PgRetentionPolicyRepository::new(pool.clone()));

        // Extra clones for CameraManager (needs its own repo handles)
        let camera_repo2 = Arc::new(PgCameraRepository::new(pool.clone()));
        let recording_repo2 = Arc::new(PgRecordingRepository::new(pool.clone()));
        let segment_repo2 = Arc::new(PgRecordingSegmentRepository::new(pool.clone()));
        let event_repo2 = Arc::new(PgDetectionEventRepository::new(pool.clone()));
        let zone_repo2 = Arc::new(PgDetectionZoneRepository::new(pool.clone()));

        // Initialize credential encryptor if env var is set
        let credential_encryptor: Option<Arc<dyn CredentialEncryptor>> = match
            open_nvr_infrastructure::crypto::credentials::CredentialEncryptor::from_env()
        {
            Ok(enc) => {
                tracing::info!("Credential encryption enabled");
                Some(Arc::new(enc))
            }
            Err(e) => {
                tracing::warn!(error = %e, "Credential encryption not available - credentials will not be encrypted");
                None
            }
        };

        let mut camera_cmd_service = CameraCommandService::new(
            camera_repo.clone() as Arc<dyn CameraRepository>,
            audit_repo.clone() as Arc<dyn AuditRepository>,
        );
        if let Some(enc) = credential_encryptor {
            camera_cmd_service = camera_cmd_service.with_encryptor(enc);
        }
        let camera_commands = Arc::new(camera_cmd_service);

        let camera_queries = Arc::new(CameraQueryService::new(
            camera_repo as Arc<dyn CameraRepository>,
        ));

        let group_commands = Arc::new(GroupCommandService::new(
            group_repo as Arc<dyn CameraGroupRepository>,
            audit_repo.clone() as Arc<dyn AuditRepository>,
        ));

        let recording_queries = Arc::new(RecordingQueryService::new(
            recording_repo.clone() as Arc<dyn RecordingRepository>,
            segment_repo as Arc<dyn RecordingSegmentRepository>,
            None, // ObjectStorage will be connected when MinIO is configured
        ));

        let timeline_queries = Arc::new(TimelineQueryService::new(
            recording_repo as Arc<dyn RecordingRepository>,
            event_repo.clone() as Arc<dyn DetectionEventRepository>,
        ));

        let (live_tx, _) = broadcast::channel(1024);

        let camera_manager = Some(Arc::new(CameraManager::new(
            camera_repo2 as Arc<dyn CameraRepository>,
            recording_repo2 as Arc<dyn RecordingRepository>,
            segment_repo2 as Arc<dyn RecordingSegmentRepository>,
            event_repo2 as Arc<dyn DetectionEventRepository>,
            zone_repo2 as Arc<dyn DetectionZoneRepository>,
            None, // ObjectStorage - will be connected when MinIO env is set
            live_tx.clone(),
        )));

        Self {
            camera_commands,
            camera_queries,
            group_commands,
            recording_queries,
            timeline_queries,
            audit_repo: audit_repo.clone() as Arc<dyn AuditRepository>,
            network_event_repo: network_event_repo as Arc<dyn NetworkEventRepository>,
            zone_repo: zone_repo as Arc<dyn DetectionZoneRepository>,
            event_repo: event_repo as Arc<dyn DetectionEventRepository>,
            layout_repo: layout_repo as Arc<dyn GridLayoutRepository>,
            retention_repo: retention_repo as Arc<dyn RetentionPolicyRepository>,
            camera_manager,
            db_pool: pool,
            live_tx,
        }
    }
}
