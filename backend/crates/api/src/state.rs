use open_nvr_application::commands::*;
use open_nvr_application::queries::*;
use open_nvr_domain::ports::*;
use open_nvr_infrastructure::persistence::*;
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
    pub audit_repo: Arc<PgAuditRepository>,
    pub network_event_repo: Arc<PgNetworkEventRepository>,
    pub zone_repo: Arc<dyn DetectionZoneRepository>,
    pub event_repo: Arc<dyn DetectionEventRepository>,
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

        let camera_commands = Arc::new(CameraCommandService::new(
            camera_repo.clone() as Arc<dyn CameraRepository>,
            audit_repo.clone() as Arc<dyn AuditRepository>,
        ));

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

        Self {
            camera_commands,
            camera_queries,
            group_commands,
            recording_queries,
            timeline_queries,
            audit_repo,
            network_event_repo,
            zone_repo: zone_repo as Arc<dyn DetectionZoneRepository>,
            event_repo: event_repo as Arc<dyn DetectionEventRepository>,
            db_pool: pool,
            live_tx,
        }
    }
}
