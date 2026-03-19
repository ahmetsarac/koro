use axum::{Json, extract::State, http::StatusCode};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    users::{
        models::{MeResponse, SetupRequest, SetupResponse},
        service as users_service,
    },
};

pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<(StatusCode, Json<MeResponse>), AppError> {
    let me = users_service::get_me(&state.db, user_id).await?;
    Ok((StatusCode::OK, Json(me)))
}

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
