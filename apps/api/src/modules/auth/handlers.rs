use axum::{Json, extract::State, http::StatusCode};

use crate::modules::{
    auth::{
        models::{AuthTokensResponse, LoginInput, LoginRequest, RefreshRequest, SignupInput, SignupRequest},
        service as auth_service,
    },
    core::{state::AppState, AppError},
};

#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "JWT token pair", body = AuthTokensResponse),
        (status = 401, description = "Invalid email or password"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthTokensResponse>), AppError> {
    let secret = auth_service::require_jwt_secret()?;
    let tokens = auth_service::login(
        &state.db,
        LoginInput {
            email: req.email,
            password: req.password,
        },
        &secret,
    )
    .await?;
    Ok((StatusCode::OK, Json(tokens)))
}

#[utoipa::path(
    post,
    path = "/signup",
    tag = "auth",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "JWT token pair", body = AuthTokensResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already registered"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<(StatusCode, Json<AuthTokensResponse>), AppError> {
    let secret = auth_service::require_jwt_secret()?;
    let tokens = auth_service::signup(
        &state.db,
        SignupInput {
            email: req.email,
            name: req.name,
            password: req.password,
        },
        &secret,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(tokens)))
}

#[utoipa::path(
    post,
    path = "/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "New JWT token pair", body = AuthTokensResponse),
        (status = 401, description = "Invalid refresh token"),
        (status = 500, description = "Server error"),
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<AuthTokensResponse>), AppError> {
    let secret = auth_service::require_jwt_secret()?;
    let tokens = auth_service::refresh(&state.db, &req.refresh_token, &secret).await?;
    Ok((StatusCode::OK, Json(tokens)))
}
