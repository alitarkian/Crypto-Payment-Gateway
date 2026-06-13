use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

pub async fn connect(url: &str) -> anyhow::Result<PgPool> {
    info!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database connected and migrations applied");

    Ok(pool)
}