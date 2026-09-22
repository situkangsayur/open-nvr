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
    // The in-process RTSP recorder uploads to a null store unless object
    // storage is wired up, so all it did was hold an extra RTSP session per
    // camera (cheap cameras allow two or three) and add a `recordings` row on
    // every reconnect. Real recording is the ffmpeg MP4 archive. Opt back in
    // with LEGACY_RECORDER=1.
    let legacy_recorder = std::env::var("LEGACY_RECORDER").map(|v| v == "1").unwrap_or(false);
    let state_clone = state.clone();
    tokio::spawn(async move {
        // Start all cameras with recording enabled
        if let (true, Some(ref mgr)) = (legacy_recorder, &state_clone.camera_manager) {
            info!("Starting camera manager - loading cameras...");
            mgr.start_all().await;
            let running = mgr.running_count().await;
            info!(running = running, "Camera manager initialized");
        }
    });

    let status_repo: Arc<dyn CameraRepository> = Arc::new(
        open_nvr_infrastructure::persistence::PgCameraRepository::new(pool.clone()),
    );

    // Stream reconcile loop: keeps go2rtc, live HLS and recording running for
    // every camera in the table. Runs forever rather than once at boot, so a
    // camera that was offline at startup (or after a reboot) is picked up as
    // soon as it answers, and a dead process is replaced.
    {
        let state = state.clone();
        let status_repo = status_repo.clone();
        tokio::spawn(async move {
            if let Some(ref hls) = state.hls_manager {
                hls.set_via_go2rtc(state.go2rtc.is_some());
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            loop {
                match state.camera_queries.list_cameras().await {
                    Ok(cameras) => {
                        let restarted = match state.go2rtc {
                            Some(ref g) => g.ensure(&cameras).await,
                            None => false,
                        };
                        if let Some(ref hls) = state.hls_manager {
                            hls.check_and_restart().await;
                            hls.reconcile(&cameras, restarted).await;

                            // Status follows the stream, not a TCP probe: a
                            // camera that answers on port 554 but sends no
                            // video is not "online" to anyone watching, and a
                            // camera nobody is recording must not sit on
                            // "connecting" forever.
                            for camera in &cameras {
                                let status = hls.stream_status(&camera.id).await;
                                if status != camera.status {
                                    let _ = status_repo.update_status(camera.id, status).await;
                                }
                            }
                        }
                    }
                    Err(e) => tracing::warn!(error = %e, "Stream reconcile: cannot load cameras"),
                }
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            }
        });
    }

    // Find ONVIF PTZ endpoints (and sub streams) for every camera in the
    // background, so the UI knows which cameras can move.
    tokio::spawn(open_nvr_api::routes::ptz::discover_all(state.clone()));

    // Storage rotation: keep recordings inside the configured disk budget
    // (RECORDINGS_MAX_DISK_PERCENT, default 80%) by deleting oldest segments.
    let rotation_config = open_nvr_worker::storage_rotation::RotationConfig::from_env();
    tokio::spawn(open_nvr_worker::storage_rotation::storage_rotation_worker(
        rotation_config,
    ));

    // Health monitor: only a fallback now. The reconcile loop derives status
    // from the live stream, which is what users actually care about, so the
    // TCP probe would only fight it. Re-enable with TCP_HEALTH_MONITOR=1.
    if std::env::var("TCP_HEALTH_MONITOR").map(|v| v == "1").unwrap_or(false) {
        let health_camera_repo = Arc::new(
            open_nvr_infrastructure::persistence::PgCameraRepository::new(pool),
        );
        tokio::spawn(open_nvr_worker::health::camera_health_monitor(
            health_camera_repo as Arc<dyn CameraRepository>,
        ));
    }

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
