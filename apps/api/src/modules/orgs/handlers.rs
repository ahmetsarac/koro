use axum::{Json, extract::State, http::StatusCode};

use crate::modules::{
    auth::user::AuthUser,
    core::{state::AppState, AppError},
    orgs::{
        models::{CreateOrgInput, CreateOrgRequest, CreateOrgResult},
        service as orgs_service,
    },
};

pub async fn create_org(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateOrgRequest>,
) -> Result<(StatusCode, Json<CreateOrgResult>), AppError> {
    let created = orgs_service::create_org(
        &state.db,
        user_id,
        CreateOrgInput {
            name: &req.name,
            slug: &req.slug,
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(created)))
}
