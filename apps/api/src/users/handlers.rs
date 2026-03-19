use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::{
    auth::user::AuthUser,
    core::state::AppState,
    error::AppError,
    users::service as users_service,
};

#[derive(Deserialize)]
pub struct SetupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<users_service::MeResponse>), AppError> {
    let me = users_service::get_me(&state.db, user_id).await?;
    Ok((StatusCode::OK, Json(me)))
}

pub async fn setup(
    State(state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> Result<(StatusCode, Json<users_service::SetupResponse>), AppError> {
    let out = users_service::setup_platform_admin(
        &state.db,
        req.email,
        req.name,
        req.password,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(out)))
}
