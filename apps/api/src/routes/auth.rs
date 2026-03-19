use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::{
    error::AppError, services::auth as auth_service, state::AppState,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<auth_service::AuthTokensResponse>), AppError> {
    let secret = auth_service::require_jwt_secret()?;
    let tokens = auth_service::login(
        &state.db,
        auth_service::LoginInput {
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
) -> Result<(StatusCode, Json<auth_service::AuthTokensResponse>), AppError> {
    let secret = auth_service::require_jwt_secret()?;
    let tokens = auth_service::signup(
        &state.db,
        auth_service::SignupInput {
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
) -> Result<(StatusCode, Json<auth_service::AuthTokensResponse>), AppError> {
    let secret = auth_service::require_jwt_secret()?;
    let tokens = auth_service::refresh(&state.db, &req.refresh_token, &secret).await?;
    Ok((StatusCode::OK, Json(tokens)))
}
