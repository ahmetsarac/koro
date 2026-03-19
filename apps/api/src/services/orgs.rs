use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::AppError, repos::orgs as orgs_repo};

pub struct CreateOrgInput<'a> {
    pub name: &'a str,
    pub slug: &'a str,
}

#[derive(Serialize)]
pub struct CreateOrgResult {
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
}

pub async fn create_org(
    pool: &PgPool,
    user_id: Uuid,
    input: CreateOrgInput<'_>,
) -> Result<CreateOrgResult, AppError> {
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(?e, "create_org begin tx");
        AppError::Internal
    })?;

    let (org_id, name, slug) =
        match orgs_repo::insert_organization(&mut tx, input.name, input.slug, user_id).await {
            Ok(row) => row,
            Err(e) => {
                tracing::error!(?e, "create_org insert");
                return Err(AppError::BadRequest(None));
            }
        };

    orgs_repo::insert_org_admin_member(&mut tx, org_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_org insert org_members");
            AppError::Internal
        })?;

    tx.commit().await.map_err(|e| {
        tracing::error!(?e, "create_org commit");
        AppError::Internal
    })?;

    Ok(CreateOrgResult {
        org_id,
        name,
        slug,
    })
}
