use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    invites::{
        models::{
            AcceptInviteInput, AcceptInviteRequest, AcceptInviteResponse, CreateInviteRequest,
            CreateInviteResponse, GetInviteResponse,
        },
        service as invites_service,
    },
};

pub async fn create_invite(
    Path(org_id): Path<uuid::Uuid>,
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateInviteRequest>,
) -> Result<(StatusCode, Json<CreateInviteResponse>), AppError> {
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
) -> Result<(StatusCode, Json<GetInviteResponse>), AppError> {
    let res = invites_service::get_invite(&state.db, &token).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn accept_invite(
    Path(token): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<AcceptInviteRequest>,
) -> Result<(StatusCode, Json<AcceptInviteResponse>), AppError> {
    let res = invites_service::accept_invite(
        &state.db,
        &token,
        AcceptInviteInput {
            name: req.name,
            password: req.password,
        },
    )
    .await?;
    Ok((StatusCode::OK, Json(res)))
}
