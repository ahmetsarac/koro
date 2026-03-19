use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::password,
    error::AppError,
    invites::{crypto, repository as invites_repo},
};

const INVITE_URL_PREFIX: &str = "http://localhost:3001/invites/";

#[derive(Serialize)]
pub struct CreateInviteResponse {
    pub invite_url: String,
}

pub async fn create_invite(
    pool: &PgPool,
    org_id: Uuid,
    user_id: Uuid,
    email: String,
    role: String,
) -> Result<CreateInviteResponse, AppError> {
    let is_admin = invites_repo::is_org_admin(pool, org_id, user_id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "create_invite admin check");
            AppError::Internal
        })?;

    if !is_admin {
        return Err(AppError::Forbidden);
    }

    let raw_token = crypto::generate_token();
    let token_hash = crypto::hash_token(&raw_token);
    let expires_at = Utc::now() + Duration::days(7);

    invites_repo::insert_invite(
        pool,
        org_id,
        &email,
        &role,
        &token_hash,
        expires_at,
        user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(?e, "create_invite insert");
        AppError::BadRequest(None)
    })?;

    Ok(CreateInviteResponse {
        invite_url: format!("{INVITE_URL_PREFIX}{raw_token}"),
    })
}

#[derive(Serialize)]
pub struct GetInviteResponse {
    pub org_name: String,
    pub email: String,
    pub role: String,
    pub expires_at: String,
}

pub async fn get_invite(pool: &PgPool, token: &str) -> Result<GetInviteResponse, AppError> {
    let token_hash = crypto::hash_token(token);

    let row = invites_repo::find_invite_preview_by_hash(pool, &token_hash)
        .await
        .map_err(|e| {
            tracing::error!(?e, "get_invite query");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    if row.used_at.is_some() || row.expires_at < Utc::now() {
        return Err(AppError::Gone);
    }

    Ok(GetInviteResponse {
        org_name: row.org_name,
        email: row.email,
        role: row.invited_role,
        expires_at: row.expires_at.to_rfc3339(),
    })
}

pub struct AcceptInviteInput {
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AcceptInviteResponse {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub org_role: String,
}

pub async fn accept_invite(
    pool: &PgPool,
    token: &str,
    input: AcceptInviteInput,
) -> Result<AcceptInviteResponse, AppError> {
    if input.password.len() < 8 {
        return Err(AppError::BadRequest(Some(
            "password must be at least 8 chars",
        )));
    }

    let token_hash = crypto::hash_token(token);

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!(?e, "accept_invite begin");
        AppError::Internal
    })?;

    let inv = invites_repo::fetch_invite_for_update(&mut tx, &token_hash)
        .await
        .map_err(|e| {
            tracing::error!(?e, "accept_invite fetch invite");
            AppError::Internal
        })?
        .ok_or(AppError::NotFound)?;

    if inv.used_at.is_some() || inv.expires_at < Utc::now() {
        return Err(AppError::Gone);
    }

    let existing_user = invites_repo::find_active_user_by_email_tx(&mut tx, &inv.email)
        .await
        .map_err(|e| {
            tracing::error!(?e, "accept_invite user lookup");
            AppError::Internal
        })?;

    let (user_id, had_password) = match existing_user {
        Some(u) => (u.id, u.password_hash.is_some()),
        None => {
            let id = invites_repo::insert_minimal_user_tx(&mut tx, &inv.email, &input.name)
                .await
                .map_err(|e| {
                    tracing::error!(?e, "accept_invite create user");
                    AppError::Internal
                })?;
            (id, false)
        }
    };

    if !had_password {
        let hash = password::hash_password(&input.password).map_err(|e| {
            tracing::error!(?e, "accept_invite hash");
            AppError::Internal
        })?;
        invites_repo::set_user_password_and_name_tx(&mut tx, user_id, &hash, &input.name)
            .await
            .map_err(|e| {
                tracing::error!(?e, "accept_invite set password");
                AppError::Internal
            })?;
    }

    invites_repo::insert_org_member_on_conflict_nothing_tx(
        &mut tx,
        inv.org_id,
        user_id,
        &inv.invited_role,
    )
    .await
    .map_err(|e| {
        tracing::error!(?e, "accept_invite insert org_member");
        AppError::Internal
    })?;

    invites_repo::mark_invite_used_tx(&mut tx, inv.id)
        .await
        .map_err(|e| {
            tracing::error!(?e, "accept_invite mark used");
            AppError::Internal
        })?;

    tx.commit().await.map_err(|e| {
        tracing::error!(?e, "accept_invite commit");
        AppError::Internal
    })?;

    Ok(AcceptInviteResponse {
        user_id,
        org_id: inv.org_id,
        org_role: inv.invited_role,
    })
}
