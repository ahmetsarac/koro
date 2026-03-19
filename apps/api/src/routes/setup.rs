use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::{
    error::AppError, services::setup as setup_service, state::AppState,
};

#[derive(Deserialize)]
pub struct SetupRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

pub async fn setup(
    State(state): State<AppState>,
    Json(req): Json<SetupRequest>,
) -> Result<(StatusCode, Json<setup_service::SetupResponse>), AppError> {
    let out = setup_service::setup_platform_admin(
        &state.db,
        req.email,
        req.name,
        req.password,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(out)))
}
