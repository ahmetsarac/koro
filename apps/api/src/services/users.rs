use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, repos::users as users_repo};

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
