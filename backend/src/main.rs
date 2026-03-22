use open_nvr_api::routes::create_router;
use open_nvr_api::state::AppState;
use open_nvr_infrastructure::persistence;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
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

    let state = AppState::new(pool);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(state)
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
