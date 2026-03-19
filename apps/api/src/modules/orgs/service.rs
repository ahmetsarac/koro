use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::{core::AppError, orgs::models::*, orgs::repository as orgs_repo};

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
