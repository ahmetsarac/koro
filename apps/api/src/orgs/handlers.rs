use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;

use crate::{
    auth::user::AuthUser,
    core::state::AppState,
    error::AppError,
    orgs::service as orgs_service,
};

#[derive(Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
}

pub async fn create_org(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<orgs_service::CreateOrgResult>), AppError> {
    let created = orgs_service::create_org(
        &state.db,
        user_id,
        orgs_service::CreateOrgInput {
            name: &req.name,
            slug: &req.slug,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(created)))
}
