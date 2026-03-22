use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::PathBuf;
use tracing::{info, warn};

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    info!("Database connection pool created");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let possible_paths: Vec<PathBuf> = vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../migrations"),
        PathBuf::from("./migrations"),
        PathBuf::from("../migrations"),
    ];

    for path in &possible_paths {
        if path.exists() {
            let canonical = path.canonicalize()?;
            info!(path = %canonical.display(), "Running migrations");
            let migrator: sqlx::migrate::Migrator = sqlx::migrate::Migrator::new(canonical).await?;
            migrator.run(pool).await?;
            info!("Database migrations applied");
            return Ok(());
        }
    }

    warn!("No migrations directory found, skipping migrations");
    Ok(())
}
