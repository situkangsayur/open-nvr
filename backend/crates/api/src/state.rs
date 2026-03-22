use open_nvr_application::commands::*;
use open_nvr_application::queries::*;
use open_nvr_domain::ports::*;
use open_nvr_infrastructure::persistence::*;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub camera_commands: Arc<CameraCommandService>,
    pub camera_queries: Arc<CameraQueryService>,
    pub group_commands: Arc<GroupCommandService>,
    pub audit_repo: Arc<PgAuditRepository>,
    pub network_event_repo: Arc<PgNetworkEventRepository>,
    pub db_pool: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let camera_repo = Arc::new(PgCameraRepository::new(pool.clone()));
        let group_repo = Arc::new(PgCameraGroupRepository::new(pool.clone()));
        let audit_repo = Arc::new(PgAuditRepository::new(pool.clone()));
        let network_event_repo = Arc::new(PgNetworkEventRepository::new(pool.clone()));

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

        Self {
            camera_commands,
            camera_queries,
            group_commands,
            audit_repo,
            network_event_repo,
            db_pool: pool,
        }
    }
}
