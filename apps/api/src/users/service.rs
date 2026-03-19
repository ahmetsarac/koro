use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{auth::password, error::AppError, users::repository as users_repo};

#[derive(Serialize)]
pub struct UserOrganization {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub organizations: Vec<UserOrganization>,
}

pub async fn get_me(pool: &PgPool, user_id: Uuid) -> Result<MeResponse, AppError> {
    let user = users_repo::find_user_by_id(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_me user query");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let org_rows = users_repo::list_org_memberships(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_me orgs query");
            AppError::Internal
        })?;

    let organizations = org_rows
        .into_iter()
        .map(|r| UserOrganization {
            id: r.id,
            name: r.name,
            slug: r.slug,
            role: r.org_role,
        })
        .collect();

    Ok(MeResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        organizations,
    })
}

// --- Platform setup (first admin) ------------------------------------------

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
        return Err(AppError::Conflict(None));
    }

    let password_hash = password::hash_password(&password).map_err(|e| {
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
