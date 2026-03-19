use axum::{Json, extract::State, http::StatusCode};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    users::{
        models::{MeResponse, SetupRequest, SetupResponse},
        service as users_service,
    },
};

#[utoipa::path(
    get,
    path = "/me",
    tag = "users",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current user + orgs", body = MeResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<MeResponse>), AppError> {
    let me = users_service::get_me(&state.db, user_id).await?;
    Ok((StatusCode::OK, Json(me)))
}

#[utoipa::path(
    post,
    path = "/setup",
    tag = "users",
    request_body = SetupRequest,
    responses(
        (status = 201, description = "Platform admin created", body = SetupResponse),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Setup not allowed"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn setup(
    State(state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> Result<(StatusCode, Json<SetupResponse>), AppError> {
    let out = users_service::setup_platform_admin(
        &state.db,
        req.email,
        req.name,
        req.password,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(out)))
}
