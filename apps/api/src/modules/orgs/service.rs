use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::{
    core::AppError,
    invites::repository as invites_repo,
    orgs::models::*,
    orgs::repository as orgs_repo,
};

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

pub async fn patch_org(
    pool: &PgPool,
    org_slug: &str,
    user_id: Uuid,
    name: &str,
) -> Result<PatchOrgResponse, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(Some("name is required")));
    }
    if trimmed.len() > 200 {
        return Err(AppError::BadRequest(Some("name is too long")));
    }

    let org_id = orgs_repo::find_org_id_by_slug(pool, org_slug)
        .await
        .map_err(|e| {
            tracing::error!(?e, "patch_org org resolve");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    let is_admin = invites_repo::is_org_admin(pool, org_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "patch_org admin check");
            AppError::Internal
        })?;
    if !is_admin {
        return Err(AppError::Forbidden);
    }

    let updated = orgs_repo::update_organization_name(pool, org_id, trimmed)
        .await
        .map_err(|e| {
            tracing::error!(?e, "patch_org update");
            AppError::Internal
        })?;
    let Some((id, name, slug)) = updated else {
        return Err(AppError::NotFound);
    };

    Ok(PatchOrgResponse { id, name, slug })
}
