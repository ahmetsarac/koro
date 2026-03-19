use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{
    auth::user::AuthUser,
    core::state::AppState,
    error::AppError,
    invites::service as invites_service,
};

#[derive(Deserialize)]
pub struct CreateInviteRequest {
    pub email: String,
    pub role: String,
}

pub async fn create_invite(
    Path(org_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<invites_service::CreateInviteResponse>), AppError> {
    let res = invites_service::create_invite(
        &state.db,
        org_id,
        user_id,
        req.email,
        req.role,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn get_invite(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<invites_service::GetInviteResponse>), AppError> {
    let res = invites_service::get_invite(&state.db, &token).await?;
    Ok((StatusCode::OK, Json(res)))
}

#[derive(Deserialize)]
pub struct AcceptInviteRequest {
    pub name: String,
    pub password: String,
}

pub async fn accept_invite(
    Path(token): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<AcceptInviteRequest>,
) -> Result<(StatusCode, Json<invites_service::AcceptInviteResponse>), AppError> {
    let res = invites_service::accept_invite(
        &state.db,
        &token,
        invites_service::AcceptInviteInput {
            name: req.name,
            password: req.password,
        },
    )
    .await?;
    Ok((StatusCode::OK, Json(res)))
}
