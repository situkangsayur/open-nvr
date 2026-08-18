use axum::middleware as axum_middleware;
use open_nvr_api::middleware::auth::KeycloakConfig;
use open_nvr_api::middleware::security_headers::security_headers_middleware;
use open_nvr_api::routes::create_router;
use open_nvr_api::state::AppState;
use open_nvr_domain::ports::CameraRepository;
use open_nvr_infrastructure::persistence;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use axum::http::{HeaderName, HeaderValue, Method};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    info!("Starting Open-NVR server");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://opennvr:opennvr_secret@localhost:5432/opennvr".into());

    let pool = persistence::create_pool(&database_url).await?;

    persistence::run_migrations(&pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;

    info!("Database migrations applied successfully");

    let state = AppState::new(pool.clone());

    // Initialize NATS broker (optional)
    let nats = open_nvr_infrastructure::messaging::nats_broker::OptionalNats::from_env().await;
    if nats.is_connected() {
        info!("NATS message broker connected");
    } else {
        info!("Running without NATS message broker");
    }

    // Start background workers
    let state_clone = state.clone();
    tokio::spawn(async move {
        // Start all cameras with recording enabled
        if let Some(ref mgr) = state_clone.camera_manager {
            info!("Starting camera manager - loading cameras...");
            mgr.start_all().await;
            let running = mgr.running_count().await;
            info!(running = running, "Camera manager initialized");
        }
    });

    // Start HLS streams for online cameras
    if let Some(ref hls_mgr) = state.hls_manager {
        let cameras = state.camera_queries.list_cameras().await.unwrap_or_default();
        let online: Vec<_> = cameras
            .iter()
            .filter(|c| c.status == open_nvr_domain::entities::CameraStatus::Online)
            .cloned()
            .collect();
        if !online.is_empty() {
            let mgr = hls_mgr.clone();
            tokio::spawn(async move {
                // Wait for cameras to be fully online
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                mgr.start_all(&online).await;
                let count = mgr.active_count().await;
                info!(count = count, "HLS streams started");
            });
        }
    }

    // HLS watchdog: restart dead/hung ffmpeg processes every 30s
    if let Some(ref hls_mgr) = state.hls_manager {
        let watchdog_mgr = hls_mgr.clone();
        tokio::spawn(async move {
            // Wait for initial streams to start
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            loop {
                watchdog_mgr.check_and_restart().await;
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            }
        });
    }

    // Storage rotation: keep recordings inside the configured disk budget
    // (RECORDINGS_MAX_DISK_PERCENT, default 80%) by deleting oldest segments.
    let rotation_config = open_nvr_worker::storage_rotation::RotationConfig::from_env();
    tokio::spawn(open_nvr_worker::storage_rotation::storage_rotation_worker(
        rotation_config,
    ));

    // Health monitor
    let health_camera_repo = Arc::new(
        open_nvr_infrastructure::persistence::PgCameraRepository::new(pool),
    );
    tokio::spawn(open_nvr_worker::health::camera_health_monitor(
        health_camera_repo as Arc<dyn CameraRepository>,
    ));

    info!("Background workers started");

    let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
            HeaderName::from_static("x-request-id"),
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600));

    // Keycloak public key for /api token validation. Fetching it here is only a
    // warm-up: the middleware fetches on demand if Keycloak was not up yet, so
    // a cold Keycloak must not stop the NVR from booting and recording.
    let keycloak_config = KeycloakConfig::new(
        &std::env::var("KEYCLOAK_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
        &std::env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "opennvr".into()),
    );
    match keycloak_config.fetch_public_key().await {
        Ok(()) => info!("Keycloak signing key loaded"),
        Err(e) => tracing::warn!(error = %e, "Keycloak key not loaded yet; will retry per request"),
    }

    let app = create_router(state)
        .layer(axum::Extension(keycloak_config))
        .layer(axum_middleware::from_fn(security_headers_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let bind_addr: SocketAddr = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:8888".into())
        .parse()?;

    info!(%bind_addr, "Server starting");

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
