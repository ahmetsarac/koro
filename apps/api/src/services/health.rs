use sqlx::PgPool;

use crate::db;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Ok,
    DbDown,
}

pub async fn check(pool: &PgPool) -> HealthStatus {
    match db::ping(pool).await {
        Ok(()) => HealthStatus::Ok,
        Err(err) => {
            tracing::error!(?err, "healthcheck failed");
            HealthStatus::DbDown
        }
    }
}
