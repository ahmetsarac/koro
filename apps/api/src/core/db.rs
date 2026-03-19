use anyhow::Context;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    println!("Database URL: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
        .context("failed to connect to Postgres")?;

    Ok(pool)
}

pub async fn ping(pool: &PgPool) -> anyhow::Result<()> {
    // minimal DB check
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .context("db ping failed")?;

    Ok(())
}
