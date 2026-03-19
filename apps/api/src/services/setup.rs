use serde::Serialize;
use sqlx::PgPool;

use crate::{auth, error::AppError, repos::users as users_repo};

#[derive(Serialize)]
pub struct SetupResponse {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub platform_role: &'static str,
}

pub async fn setup_platform_admin(
    pool: &PgPool,
    email: String,
    name: String,
    password: String,
) -> Result<SetupResponse, AppError> {
    if password.len() < 8 {
        return Err(AppError::BadRequest(Some(
            "password must be at least 8 chars",
        )));
    }

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(?e, "setup begin tx");
        AppError::Internal
    })?;

    let user_count = users_repo::count_users_tx(&mut tx).await.map_err(|e| {
        tracing::error!(?e, "setup count users");
        AppError::Internal
    })?;

    if user_count > 0 {
        return Err(AppError::Conflict);
    }

    let password_hash = auth::hash_password(&password).map_err(|e| {
        tracing::error!(?e, "setup hash password");
        AppError::Internal
    })?;

    let (user_id, email) = users_repo::insert_platform_admin_user(
        &mut tx,
        &email,
        &name,
        &password_hash,
    )
    .await
    .map_err(|e| {
        tracing::error!(?e, "setup insert user");
        AppError::Internal
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!(?e, "setup commit");
        AppError::Internal
    })?;

    Ok(SetupResponse {
        user_id,
        email,
        platform_role: "platform_admin",
    })
}
