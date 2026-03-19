use axum::{Json, extract::State, http::StatusCode};

use crate::{
    auth::{
        models::{AuthTokensResponse, LoginInput, LoginRequest, RefreshRequest, SignupInput, SignupRequest},
        service as auth_service,
    },
    core::{state::AppState, AppError},
};

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

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<AuthTokensResponse>), AppError> {
    let secret = auth_service::require_jwt_secret()?;
    let tokens = auth_service::refresh(&state.db, &req.refresh_token, &secret).await?;
    Ok((StatusCode::OK, Json(tokens)))
}
