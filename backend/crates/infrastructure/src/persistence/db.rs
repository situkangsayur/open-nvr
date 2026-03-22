use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::Path;
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    info!("Database connection pool created");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../migrations");

    let migrator = sqlx::migrate::Migrator::new(migrations_dir).await?;
    migrator.run(pool).await?;

    info!("Database migrations applied");
    Ok(())
}
