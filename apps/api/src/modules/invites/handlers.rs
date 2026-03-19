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

#[utoipa::path(
    post,
    path = "/orgs/{orgId}/invites",
    tag = "invites",
    security(("bearer_auth" = [])),
    params(
        ("orgId" = uuid::Uuid, Path),
    ),
    request_body = CreateInviteRequest,
    responses(
        (status = 201, description = "Invite created", body = CreateInviteResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Server error"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/invites/{token}",
    tag = "invites",
    params(
        ("token" = String, Path, description = "Invite token"),
    ),
    responses(
        (status = 200, description = "Invite metadata", body = GetInviteResponse),
        (status = 404, description = "Not found or expired"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_invite(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<GetInviteResponse>), AppError> {
    let res = invites_service::get_invite(&state.db, &token).await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    post,
    path = "/invites/{token}/accept",
    tag = "invites",
    params(
        ("token" = String, Path),
    ),
    request_body = AcceptInviteRequest,
    responses(
        (status = 200, description = "Membership created", body = AcceptInviteResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Invalid invite"),
        (status = 500, description = "Server error"),
    )
)]
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
